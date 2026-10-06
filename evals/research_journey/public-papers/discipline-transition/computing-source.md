# Vaswani2017Attention — bounded source notes

Authors: Ashish Vaswani et al. Paper: Attention Is All You Need (2017).
Inspected version: arXiv:1706.03762v7, dated 2023-08-02.
Source: [HTML](https://arxiv.org/html/1706.03762v7) and
[PDF text](https://arxiv.org/pdf/1706.03762v7), read by the coordinator through
Host web on 2026-10-02. These are paraphrased source notes, not raw excerpts.
Raw response byte hashes are unavailable. The local note digest identifies only
these notes. No code, dataset or benchmark execution was inspected.

## Vaswani2017Attention:training-cost

Section 6.1 (PDF p.8) estimates training FLOPs from duration, GPU count and
estimated sustained GPU capacity. This passage does not report inference timing.

## Vaswani2017Attention:result-scope

Table 2 (PDF p.8) uses newstest2014; Table 3 (p.9) uses newstest2013 development
data. Section 6.2 says the latter omits checkpoint averaging. For big-model
English–French BLEU, the abstract and Table 2 give 41.8, while Section 6.1 gives
41.0 in both retrieved text representations. The cause is unresolved; no
publisher correction or matching code output was inspected.
