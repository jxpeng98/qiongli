mod tests {
    use agent_client_protocol::schema::{
        ProtocolVersion,
        v1::{
            ContentBlock, ContentChunk, InitializeRequest, InitializeResponse, NewSessionRequest,
            NewSessionResponse, PromptRequest, PromptResponse, SessionNotification, SessionUpdate,
            StopReason, TextContent,
        },
    };
    use agent_client_protocol::{Agent, Client, ConnectionTo, Responder};

    #[test]
    fn direct_v1_round_trip_streams_a_provider_neutral_result() {
        futures::executor::block_on(async {
            let agent = Agent
                .builder()
                .on_receive_request(
                    async |request: InitializeRequest,
                           responder: Responder<InitializeResponse>,
                           _connection: ConnectionTo<Client>| {
                        assert_eq!(request.protocol_version, ProtocolVersion::V1);
                        responder.respond(InitializeResponse::new(ProtocolVersion::V1))
                    },
                    agent_client_protocol::on_receive_request!(),
                )
                .on_receive_request(
                    async |_request: NewSessionRequest,
                           responder: Responder<NewSessionResponse>,
                           _connection: ConnectionTo<Client>| {
                        responder.respond(NewSessionResponse::new("fixture-session"))
                    },
                    agent_client_protocol::on_receive_request!(),
                )
                .on_receive_request(
                    async |request: PromptRequest,
                           responder: Responder<PromptResponse>,
                           connection: ConnectionTo<Client>| {
                        connection.send_notification(SessionNotification::new(
                            request.session_id,
                            SessionUpdate::AgentMessageChunk(ContentChunk::new(
                                ContentBlock::Text(TextContent::new("fixture reply")),
                            )),
                        ))?;
                        responder.respond(PromptResponse::new(StopReason::EndTurn))
                    },
                    agent_client_protocol::on_receive_request!(),
                );

            let outcome = Client
                .builder()
                .name("qiongli-acp-v1-fixture")
                .connect_with(agent, async |connection| {
                    let initialized = connection
                        .send_request(InitializeRequest::new(ProtocolVersion::V1))
                        .block_task()
                        .await?;
                    assert_eq!(initialized.protocol_version, ProtocolVersion::V1);

                    let mut session = connection
                        .build_session_cwd()?
                        .block_task()
                        .start_session()
                        .await?;
                    let session_id = session.session_id().to_string();
                    session.send_prompt("fixture prompt")?;
                    let assistant_text = session.read_to_string().await?;

                    Ok((
                        initialized.protocol_version.as_u16(),
                        session_id,
                        assistant_text,
                    ))
                })
                .await
                .expect("credential-free ACP v1 fixture should complete");

            assert_eq!(
                outcome,
                (1, "fixture-session".into(), "fixture reply".into())
            );
        });
    }
}
