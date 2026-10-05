"""Offline public/synthetic AGY capture regressions; never run a real Host."""
import copy
import hashlib
import json
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

from evals.research_journey import antigravity_observation as agy

SERVER = "qiongli_qiongli"
CWD = "/Public/CaseSensitive/Workspace"


def envelope():
    providers = {name: "missing" for name in ("openalex", "semantic_scholar", "crossref", "pubmed", "arxiv")}
    body = {"status": "ok", "config_path": "/Synthetic/config.json", "providers": providers,
            "capability_mode": "strategy_only", "missing": [], "redacted_config": {"providers": {
                name: {"enabled": False, "configured": False, "fields": {}} for name in providers}}}
    return {"content": [{"type": "text", "text": json.dumps(body)}], "structuredContent": body,
            "isError": False}


def events(output=None):
    parameters = {"ServerName": SERVER, "ToolName": agy.TOOL, "Arguments": {"cwd": CWD}}
    updates = []
    for state in ("ACTIVE", "DONE"):
        info = {"name": "call_mcp_tool", "parameters": parameters}
        if state == "DONE":
            info["output"] = json.dumps(envelope() if output is None else output)
        updates.append({"event": "step_update", "step_update": {
            "conversation_id": "synthetic", "step_index": 2, "state": state,
            "step_type": "tool", "tool_name": "call_mcp_tool", "tool_info": info}})
    return [{"event": "init", "conversation_id": "synthetic", "init": {"cwd": CWD}}, *updates,
            {"event": "result", "result": {"conversation_id": "synthetic", "status": "SUCCESS",
             "denied_actions": [], "num_turns": 1, "response": "Synthetic observed status.", "usage": {"input_tokens": 1}}}]


def inspect(value):
    return agy.inspect_stream("\n".join(map(json.dumps, value)), SERVER, CWD)


def menu():
    tool = SERVER + "/" + agy.TOOL
    return (tool + '(' + json.dumps({"cwd": CWD}).replace("CaseSensitive", "Case\nSensitive")
            + ') (ctrl+o to collapse)\nAllow calling this tool?\n> 1. Yes, allow tool call\n'
            + f"2. Yes, and always allow tool '{tool}' in this conversation\n"
            + f"3. Yes, and always allow tool '{tool}' (Persist to settings.json)\n4. No, deny tool call\n")


class AntigravityObservationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def test_exact_once_menu_preserves_wrapped_case_sensitive_cwd(self):
        self.assertTrue(agy.once_permission(menu(), SERVER, CWD))
        for value in (menu().replace('> 1.', '  1.'), menu().replace('config_status', 'other_tool'),
                      menu().replace('Workspace', 'workspace'), menu().replace('Case\nSensitive', 'casesensitive'),
                      menu().replace('"cwd":', '"other":'), menu().replace('Persist to settings.json', 'Allow all')):
            with self.subTest(value=value):
                self.assertFalse(agy.once_permission(value, SERVER, CWD))

    def test_full_schema_envelope_and_raw_value_bindings(self):
        result = inspect(events())
        self.assertTrue(all(result['checks'].values()))
        call = result['calls'][0]
        raw = events()[2]['step_update']['tool_info']
        for key in ('parameters', 'output'):
            expected = hashlib.sha256(json.dumps(raw[key], ensure_ascii=False, sort_keys=True,
                                                separators=(',', ':')).encode()).hexdigest()
            self.assertEqual(call[key + '_sha256'], expected)
        self.assertEqual((call['started_event'], call['completed_event']), (1, 2))

    def test_summary_missing_error_and_inconsistent_content_cannot_be_full(self):
        bad = envelope()
        bad['content'][0]['text'] = '{}'
        for value in ('Returned configuration summary', {}, envelope()['structuredContent'], bad):
            with self.subTest(value=value):
                self.assertFalse(inspect(events(value))['checks']['full_native_result'])
        for mutation in ('missing', 'error', 'denied'):
            value = events()
            info = value[2]['step_update']['tool_info']
            if mutation == 'missing':
                info.pop('output')
            elif mutation == 'error':
                info['error'] = 'synthetic error'
            else:
                value[-1]['result']['denied_actions'] = [{'action': 'mcp'}]
            self.assertFalse(all(inspect(value)['checks'].values()))

    def test_unpaired_duplicate_foreign_and_truncated_streams_fail(self):
        for mutation in ('no-start', 'duplicate', 'foreign', 'arguments', 'wrong-tool', 'pending'):
            value = copy.deepcopy(events())
            if mutation == 'no-start':
                value.pop(1)
            elif mutation == 'duplicate':
                value.insert(3, copy.deepcopy(value[2]))
            elif mutation == 'foreign':
                value[2]['step_update']['conversation_id'] = 'other'
            elif mutation == 'arguments':
                value[2]['step_update']['tool_info']['parameters']['Arguments']['cwd'] = '/wrong'
            elif mutation == 'wrong-tool':
                for event in value[1:3]:
                    event['step_update']['tool_info']['parameters']['ToolName'] = 'other'
            else:
                value.pop(2)
            with self.subTest(mutation=mutation):
                self.assertFalse(all(inspect(value)['checks'].values()))
        raw = '\n'.join(map(json.dumps, events())) + '\n{"event":'
        self.assertFalse(agy.inspect_stream(raw, SERVER, CWD)['checks']['complete_stream'])

    def run_fake(self, code, timeout=0.3):
        output = self.root / str(len(list(self.root.iterdir())))
        output.mkdir()
        result = agy.run_process([sys.executable, '-c', code], self.root, output, timeout)
        self.assertEqual(result['events_sha256'], hashlib.sha256((output / 'events.jsonl').read_bytes()).hexdigest())
        self.assertTrue(result['process_cleanup']['reaped'])
        return result, output

    def test_normal_process_spools_entire_stdout(self):
        result, output = self.run_fake("import sys;sys.stdout.write('before\\nafter\\n')", 2)
        self.assertEqual((output / 'events.jsonl').read_bytes(), b'before\nafter\n')
        self.assertEqual(result['termination_reason'], 'completed')
        self.assertEqual(result['process_cleanup']['exit_code'], 0)
        with self.assertRaises(FileExistsError):
            agy.run_process([sys.executable, '-c', 'pass'], self.root, output, 1)

    def test_timeout_terminates_and_reaps_sigterm_handler(self):
        code = "import signal,time,sys;signal.signal(signal.SIGTERM,lambda *_:sys.exit(0));print('ready',flush=True);time.sleep(30)"
        result, output = self.run_fake(code)
        self.assertEqual(result['termination_reason'], 'timeout')
        self.assertEqual(result['process_cleanup']['method'], 'sigterm')
        self.assertEqual(result['process_cleanup']['exit_code'], 0)
        self.assertIn(b'ready', (output / 'events.jsonl').read_bytes())

    def test_ignoring_sigterm_requires_kill_and_reap(self):
        process = subprocess.Popen([sys.executable, '-c',
            "import signal,time;signal.signal(signal.SIGTERM,signal.SIG_IGN);print('ready',flush=True);time.sleep(30)"],
            stdout=subprocess.PIPE, start_new_session=True)
        self.addCleanup(lambda: agy.stop_process(process, 0.1))
        self.assertEqual(process.stdout.readline(), b'ready\n')
        result = agy.stop_process(process, 0.1)
        process.stdout.close()
        self.assertEqual(result, {'exit_code': -signal.SIGKILL, 'method': 'sigkill', 'reaped': True, 'group_gone': True})

    def test_driver_exception_terminates_before_wait_and_reaps(self):
        real_wait = subprocess.Popen.wait
        def wait(process, *args, **kwargs):
            if not getattr(process, '_injected', False):
                process._injected = True
                raise RuntimeError('synthetic driver failure')
            return real_wait(process, *args, **kwargs)
        with patch.object(subprocess.Popen, 'wait', wait):
            result, _ = self.run_fake('import time;time.sleep(30)', 2)
        self.assertEqual(result['termination_reason'], 'driver-error')
        self.assertEqual(result['driver_error'], 'RuntimeError')
        self.assertIn(result['process_cleanup']['method'], ('sigterm', 'already-exited'))


    def synthetic_capture(self, *, drift=False):
        workspace = self.root / 'workspace'
        config = self.root / 'config'
        workspace.mkdir()
        config.mkdir()
        (workspace / 'source.md').write_text('Public synthetic fixture')
        identity = self.root / 'identity.json'
        identity.write_text(json.dumps({'agy_version': 'synthetic', 'source_commit': 'a' * 40,
            'cli_sha256': 'b' * 64, 'content_pack_sha256': 'c' * 64,
            'plugin_receipt_sha256': 'd' * 64, 'mcp_server': SERVER}))
        value = events()
        value[0]['init']['cwd'] = str(workspace)
        for event in value[1:3]:
            event['step_update']['tool_info']['parameters']['Arguments']['cwd'] = str(workspace)
        executable = self.root / 'fake-agy'
        code = '#!' + sys.executable + '\nimport json,os,pathlib\n'
        code += 'assert os.environ["QIONGLI_PROJECT_ROOT"]==' + repr(str(workspace)) + '\n'
        code += 'assert os.environ["QIONGLI_CONFIG_HOME"]==' + repr(str(config)) + '\n'
        if drift:
            code += 'pathlib.Path("source.md").write_text("changed synthetic fixture")\n'
        code += 'events=json.loads(' + repr(json.dumps(value)) + ')\n'
        code += 'for event in events: print(json.dumps(event),flush=True)\n'
        executable.write_text(code)
        executable.chmod(0o755)
        output = self.root / 'capture'
        receipt = agy.capture(output, workspace, config, identity, executable, 2)
        return output, receipt

    def reseal(self, capture):
        manifest_path = capture / 'manifest.json'
        manifest = json.loads(manifest_path.read_text())
        manifest['files'] = {name: hashlib.sha256((capture / name).read_bytes()).hexdigest()
                             for name in manifest['files']}
        manifest_path.write_text(json.dumps(manifest))

    def test_synthetic_capture_binds_env_and_scores_with_existing_truth_owner(self):
        capture, receipt = self.synthetic_capture()
        self.assertTrue(receipt['snapshots_unchanged'])
        original = {p.name: p.read_bytes() for p in capture.iterdir()}
        self.assertTrue(agy.score(capture, self.root / 'report'))
        self.assertEqual(original, {p.name: p.read_bytes() for p in capture.iterdir()})
        inputs = json.loads((capture / 'inputs.json').read_text())
        self.assertEqual(inputs['environment_selection']['QIONGLI_PROJECT_ROOT'], str(self.root / 'workspace'))
        with self.assertRaises(FileExistsError):
            agy.capture(capture, self.root / 'workspace', self.root / 'config', self.root / 'identity.json',
                        self.root / 'fake-agy', 2)
        self.assertEqual(original, {p.name: p.read_bytes() for p in capture.iterdir()})

    def test_snapshot_drift_never_passes(self):
        capture, receipt = self.synthetic_capture(drift=True)
        self.assertFalse(receipt['snapshots_unchanged'])
        self.assertFalse(agy.score(capture, self.root / 'report'))

    def test_missing_snapshots_cannot_be_replaced_by_a_true_flag(self):
        capture, _ = self.synthetic_capture()
        for name in ('capture.json', 'inputs.json'):
            path = capture / name
            value = json.loads(path.read_text())
            value.pop('before', None)
            value.pop('after', None)
            path.write_text(json.dumps(value))
        self.reseal(capture)
        self.assertFalse(agy.score(capture, self.root / 'report'))

    def test_changed_command_cannot_claim_preserved_model(self):
        capture, _ = self.synthetic_capture()
        path = capture / 'inputs.json'
        value = json.loads(path.read_text())
        value['command'].extend(['--model', 'unrequested'])
        path.write_text(json.dumps(value))
        self.reseal(capture)
        with self.assertRaises(ValueError):
            agy.score(capture, self.root / 'report')



    def test_boolean_process_exit_code_is_not_zero(self):
        capture, _ = self.synthetic_capture()
        path = capture / 'capture.json'
        value = json.loads(path.read_text())
        value['process_cleanup']['exit_code'] = False
        path.write_text(json.dumps(value))
        self.reseal(capture)
        self.assertFalse(agy.score(capture, self.root / 'report'))

    def test_capture_rejects_invalid_identity_before_launch(self):
        capture, _ = self.synthetic_capture()
        identity = self.root / 'identity.json'
        value = json.loads(identity.read_text())
        value['cli_sha256'] = 'invalid'
        identity.write_text(json.dumps(value))
        fresh = self.root / 'rejected-capture'
        with self.assertRaises(ValueError):
            agy.capture(fresh, self.root / 'workspace', self.root / 'config', identity, self.root / 'fake-agy', 2)
        self.assertFalse(fresh.exists())

    def test_invalid_stream_headers_cannot_pass(self):
        for mutation in ('workspace', 'missing-init', 'repeated-init', 'result-conversation'):
            value = events()
            if mutation == 'workspace':
                value[0]['init']['cwd'] = CWD.lower()
            elif mutation == 'missing-init':
                value.pop(0)
            elif mutation == 'repeated-init':
                value.insert(1, copy.deepcopy(value[0]))
            else:
                value[-1]['result']['conversation_id'] = 'foreign'
            with self.subTest(mutation=mutation):
                self.assertFalse(all(inspect(value)['checks'].values()))

    def test_wrong_env_and_snapshot_digest_cannot_pass_resealed_capture(self):
        capture, _ = self.synthetic_capture()
        path = capture / 'inputs.json'
        original = path.read_bytes()
        for mutation in ('environment', 'snapshot'):
            value = json.loads(original)
            if mutation == 'environment':
                value['environment_selection']['QIONGLI_CONFIG_HOME'] = '/wrong-config'
            else:
                value['before']['workspace']['source.md'] = 'not-a-digest'
            path.write_text(json.dumps(value))
            self.reseal(capture)
            report = self.root / ('report-' + mutation)
            with self.subTest(mutation=mutation):
                try:
                    passed = agy.score(capture, report)
                except ValueError:
                    passed = False
                self.assertFalse(passed)



    def test_completed_turn_requires_exact_integer_and_actual_response(self):
        for turns, response in ((True, 'text'), (0, 'text'), (2, 'text'), (1, ''), (1, None)):
            value = events()
            value[-1]['result'].update(num_turns=turns, response=response)
            with self.subTest(turns=turns, response=response):
                self.assertFalse(inspect(value)['checks']['single_completed_turn'])

    def test_duplicate_json_keys_do_not_authorize_or_pass_capture(self):
        duplicate = menu().replace('"cwd":', '"cwd": "/unapproved", "cwd":')
        self.assertFalse(agy.once_permission(duplicate, SERVER, CWD))
        raw = '\n'.join(map(json.dumps, events()))
        raw = raw.replace('"status": "SUCCESS"', '"status": "FAILURE", "status": "SUCCESS"')
        self.assertFalse(agy.inspect_stream(raw, SERVER, CWD)['checks']['complete_stream'])

    def test_unresolved_owned_group_prevents_a_clean_capture_pass(self):
        capture, _ = self.synthetic_capture()
        path = capture / 'capture.json'
        value = json.loads(path.read_text())
        value['process_cleanup']['group_gone'] = False
        path.write_text(json.dumps(value))
        self.reseal(capture)
        self.assertFalse(agy.score(capture, self.root / 'report'))

    def test_sigkill_exit_race_still_reaps_owned_process(self):
        from unittest.mock import Mock
        process = Mock(pid=43210)
        process.poll.side_effect = [None, 0]
        process.wait.side_effect = [subprocess.TimeoutExpired('synthetic', 0.1), 0]
        def killpg(pid, sig):
            if sig in (0, signal.SIGKILL):
                raise ProcessLookupError()
        with patch.object(agy.os, 'killpg', side_effect=killpg):
            result = agy.stop_process(process, 0.01)
        self.assertTrue(result['reaped'])
        self.assertTrue(result['group_gone'])
        self.assertEqual(result['exit_code'], 0)
        self.assertEqual(process.wait.call_count, 2)



    def test_duplicate_keys_in_serialized_envelope_or_content_are_not_full(self):
        duplicate = json.dumps(envelope()).replace('"isError": false', '"isError": true, "isError": false')
        self.assertFalse(agy.status_output(duplicate)['mcp_envelope_complete'])
        value = envelope()
        value['content'][0]['text'] = value['content'][0]['text'].replace(
            '"status": "ok"', '"status": "error", "status": "ok"')
        self.assertFalse(agy.status_output(value)['mcp_envelope_complete'])



    def test_resealed_missing_or_malformed_installation_identity_refuses(self):
        capture, _ = self.synthetic_capture()
        path = capture / 'installation.json'
        original = path.read_bytes()
        for field in ('cli_sha256', 'content_pack_sha256', 'plugin_receipt_sha256',
                      'source_commit', 'mcp_server', 'agy_version'):
            for mutation in ('missing', 'malformed'):
                value = json.loads(original)
                if mutation == 'missing':
                    value.pop(field)
                else:
                    value[field] = False
                path.write_text(json.dumps(value))
                self.reseal(capture)
                with self.subTest(field=field, mutation=mutation), self.assertRaises(ValueError):
                    agy.score(capture, self.root / ('report-' + field + '-' + mutation))



if __name__ == '__main__':
    unittest.main()
