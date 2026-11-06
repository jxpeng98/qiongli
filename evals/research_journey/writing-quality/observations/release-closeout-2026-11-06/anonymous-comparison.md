# 完整中英文示范稿：匿名对照

这是一篇基于 Doshi 与 Hauser（2024）公开论文的分析性示范稿，不是原论文的官方译文，也不报告新实验。两版均含中文和英文的摘要、引言、相关研究与定位、讨论。请将每版连贯读完，再比较语言、论证和必要限定。

来源：[Generative AI enhances individual creativity but reduces the collective diversity of novel content](https://doi.org/10.1126/sciadv.adn5290)，Science Advances，CC BY 4.0。来源全文为已保存的九页出版版本；补充材料与其引用文献没有另行评审。

M/N 在本轮重新随机分配，与此前 X/Y 或 A/B 无关。两版的来源身份和自动评审结论将在收到反馈后揭示。允许持平、各有优点，也可以保留整段原文。

请自行记录本次实际阅读和修改所花的时间；收到链接到回复之间的间隔不会被当作工作用时。不能确认的用时可写“未记录”。

## 版本 M

### 中文

#### 摘要

生成式人工智能为何能在改善个人创作的同时削弱集体多样性？本文基于Doshi和Hauser（2024）的短篇故事实验展开批判性分析，不报告新的实证实验。该研究发现，获取AI构思的机会提高了作品的新颖性与有用性评分，且收益集中于发散联想任务得分较低的作者；但同一AI实验条件内的故事彼此更为相似。〔Doshi2024 pp.3–5〕本文认为，这两项结果分别对应单篇作品的表现与作品集合内部的关系，二者并不矛盾。AI提供的构思起点既可能帮助作者改善单篇作品，也可能引导不同作者走向相近的内容方向，但其具体心理机制尚未得到单独识别。因此，评价AI的创作贡献需同时考察作品质量、收益在不同作者间的分布以及作品集合的多样性。该实验支持这一评价层次的区分，但不能据此推断长期文化多样性必然下降。

#### 引言

如果创作工具能让每位作者的作品获得更高评价，这是否意味着整个创作领域更具创造力？这一推论忽略了作品之间的关系：单篇作品的平均质量可以提高，而可供读者选择的内容差异却未必扩大。本文探讨生成式AI如何使个人创作改善与集体多样性下降并存，以及这一区分如何改变我们对AI贡献的评价方式。本文的核心论点是，单篇作品的评分不足以代表整个作品集合的创造价值。

Doshi和Hauser将作者随机分配至独立写作、可获取一个AI构思或最多获取五个AI构思三种条件。最终分析包含293篇八句短篇故事，由600名独立评估者评分，且主要评估在披露AI使用情况之前完成。〔Doshi2024 p.2〕随机分配识别的是获取AI建议机会的效果，组间差异不能解释为每多使用一个构思所带来的边际收益。由于作者仍需自行成文，该设计适合考察人机互补如何改变创作产出。本文据此区分不同的评价层次，并将作品评分的提高与作者是否获得持久创作能力明确分开。

#### 相关研究与定位

以下学术定位依据该论文对既有文献的梳理，其引用的原始文献并未另行独立审阅。Doshi和Hauser将研究接续于生成式AI提升工作质量、生产率以及人机协作叙事的相关研究，进而拓展至兼具经济价值与表达价值的创造性产出。同时，他们摒弃了比较人类与AI孰优孰劣的竞赛框架，转而关注普通参与者在AI辅助下能完成怎样的创作。〔Doshi2024 pp.1–2〕〔Doshi2024 pp.5–6〕这一定位将评价重心从机器的独立性能转向协作条件下的人类产出，使平均收益及其在不同作者间的分布同时成为核心议题。

在概念层面，论文沿用创造力研究中的新颖性与有用性两个维度，并将其具体化为故事评估指标：新颖性涵盖新颖、原创与罕见程度，有用性则涉及受众适配度、扩展成书的可行性及出版潜力。〔Doshi2024 p.1〕因此，这里的“有用性”是评估者针对特定用途作出的判断，并不等同于实际的出版成绩。作者提出，AI既能为创作提供起点，也可能通过锚定效应限制构思范围，〔Doshi2024 p.1〕并在讨论部分借用社会困境来解释个人获益与集体趋同之间的张力。〔Doshi2024 p.7〕这对本文研究问题的关键启示在于：针对单篇作品的新颖性判断，无法替代对作品集合内部差异的测量。该研究的贡献在于将个体与集体两个层次纳入同一实验加以考察，而非将其中单一指标视为创造力的完整定义。

#### 讨论

图2及正文结果显示，与独立写作组相比，最多获取五个构思组的新颖性与有用性评分分别提高了8.1%和9.0%。图3表明，这一收益主要集中于发散联想任务得分较低的作者，而高分作者并未表现出同等幅度的提升。〔Doshi2024 pp.3–4〕这一模式更支持AI能够改善较低起点作者的产出表现，而非突破最高创作上限；同时，单一任务得分也不应被视为作者全部创作潜能的定论。因此，评价平均处理效应时应保留不同受益群体之间的差异，否则仅看均值变化会掩盖AI在缩小表现差距方面的作用。

集体层面的多样性则通过另一尺度衡量。研究者计算了每篇故事的文本嵌入与同组其余故事平均嵌入之间的余弦相似度，再乘以100；图4显示，一个构思组与最多五个构思组相对独立写作基线分别增加了0.871点和0.718点。〔Doshi2024 pp.4–5〕这些数值是缩放后的相似度增量，并非多样性损失的百分比，也不意味着提供的建议越多、同质化就必然越严重。个人评分上升与集体趋同之所以能够并存，是因为作者虽相对于自身独立写作表现更佳，却共同朝相近的内容方向靠拢。评估者认为某篇故事新颖，并不意味着它在内容空间上与其他同组故事相距更远。

此外，AI条件下的故事与所提供的AI构思也更为接近，这与作者提出的锚定解释一致。〔Doshi2024 p.5〕但仅凭内容接近，尚不足以区分心理锚定与作者主动采纳有益建议这两种情形；实验采用固定提示词且不允许开放式交互，同样无法单独判定趋同究竟源于底层模型还是特定的使用方式。〔Doshi2024 pp.6–7〕因此，“提供创作支持的同时收窄构思方向”是解释上述结果的合理路径，但其具体机制仍待直接检验。短篇写作、普通参与者及单次任务的研究边界，也限制了将结论直接外推至专业创作者或长期能力变化。

基于上述分析，本文主张将作品平均质量、不同作者群体的收益分布与集合多样性并列报告；若要汇总为单一评价指标，则必须明确具体用途与权重。帮助更多作者写出合格作品，与维持作品库的内容多样性，是需要分别权衡的目标。作者提出的社会困境揭示了这两项目标可能发生冲突，但该实验并未测量长期采纳率、读者福利或文化损失的净值。〔Doshi2024 p.7〕

这一区分进一步引出可检验的后续研究设计：未来研究可对比固定建议流程与“先独立构思初稿、再引入AI辅助”的工作流，同时测量个体作品质量与集合差异，以检验能否在保留质量收益的同时减轻内容趋同；此外，还可通过后续无辅助写作任务，区分即时产出改善与可迁移的创作能力提升。这些均属于本文提出的后续检验方案，而非既有实验结论。至此，引言提出的问题得到了明确回答：生成式AI能够在改善个人作品的同时收窄作品间的差异；评价其贡献时，必须结合具体创作目标，同时考察个体表现、集体多样性及受益者分布。

### English

#### Abstract

How can generative AI improve individual creative outputs while reducing collective diversity? This article critically analyzes Doshi and Hauser’s (2024) short-story experiment without reporting new empirical data. They find that access to AI ideas increases ratings of novelty and usefulness, particularly for writers scoring lower on a divergent association task, while stories within the AI conditions become more similar to one another. 〔Doshi2024 pp.3–5〕 This article argues that these findings concern distinct properties: the quality of individual works and the relationships among works in a set. AI suggestions may help writers improve their stories while steering them toward similar content, although the underlying psychological mechanism remains unresolved. Evaluating AI’s contribution therefore requires examining output quality, the distribution of benefits across writers, and diversity across a collection. The experiment supports this analytical distinction but does not establish an inevitable decline in cultural diversity over time.

#### Introduction

If a creative tool helps every writer receive higher evaluations, does the creative field necessarily become more creative? That inference overlooks relationships among works: average quality can rise without expanding the range of distinct content available to readers. This article asks how individual improvement and reduced collective diversity can coexist, and how that distinction changes the evaluation of generative AI. Its central argument is that ratings of individual works cannot adequately represent the creative value of an entire collection.

Doshi and Hauser randomly assigned writers to work independently, access one AI idea, or access up to five ideas. Their final analysis included 293 eight-sentence stories evaluated by 600 separate participants, with the primary assessments completed before any disclosure of AI use. 〔Doshi2024 p.2〕 Randomization identifies the effect of access to suggestions; differences between conditions cannot be interpreted as the marginal benefit of each additional idea actually used. Because writers remained responsible for composing their stories, the design is suited to examining how human–AI complementarity changes output. This article uses that evidence to distinguish levels of evaluation while keeping improved story ratings separate from any claim that writers acquired lasting creative abilities.

#### Related work and positioning

The positioning developed here follows the paper’s account of prior research; the publications it cites were not independently examined. Doshi and Hauser connect their study to research on production quality, productivity, and collaborative storytelling with AI, extending that work to creative output that carries both economic and expressive value. They also distinguish their approach from comparisons asking whether humans or AI perform better, focusing instead on what ordinary participants can produce with AI assistance. 〔Doshi2024 pp.1–2〕 〔Doshi2024 pp.5–6〕 This framing shifts evaluation from the machine’s standalone capabilities to human work under collaborative conditions, making both average gains and their distribution across writers consequential.

Conceptually, the paper adopts novelty and usefulness as dimensions of creativity and adapts them to short stories. Novelty combines judgments of newness, originality, and rarity, whereas usefulness concerns suitability for the intended audience, feasibility of development into a book, and publishing potential. 〔Doshi2024 p.1〕 Usefulness thus reflects evaluators’ judgments regarding a specific purpose rather than observed publication success. The authors propose that AI can supply starting points for writing while anchoring writers’ ideas, and they later invoke a social dilemma to interpret the tension between individual benefit and collective convergence. 〔Doshi2024 p.1〕 〔Doshi2024 p.7〕 For our research question, the key insight is that judgments of novelty within individual works cannot substitute for measuring differences across a collection. The study brings both levels into a single experiment without treating either measure as an exhaustive definition of creativity.

#### Discussion

Figure 2 and the accompanying text show that access to up to five ideas increased novelty ratings by 8.1% and usefulness ratings by 9.0% relative to independent writing. Figure 3 indicates that these gains were concentrated among writers with lower divergent association task scores, whereas higher-scoring writers did not show comparable improvements. 〔Doshi2024 pp.3–4〕 This pattern supports the view that AI improves performance from a lower baseline rather than expanding the upper bound of creative achievement. Nor should a single task score be treated as a complete measure of a writer’s creative potential. Evaluations of average treatment effects should therefore retain differences across beneficiaries, as a mean shift alone can obscure AI’s role in narrowing performance gaps.

Collective diversity was assessed on a different scale. The researchers computed the cosine similarity between each story’s text embedding and the average embedding of the other stories in the same condition, multiplied by 100. Relative to independent writing, similarity increased by 0.871 points with access to one idea and by 0.718 points with access to up to five ideas, as shown in Figure 4. 〔Doshi2024 pp.4–5〕 These values are scaled similarity increments rather than percentages of diversity lost, and they do not imply that offering more suggestions necessarily produces greater homogenization. Individual improvement and collective convergence can coexist because writers may perform better than they would independently while moving toward content directions shared by peers in the same condition. An evaluator’s judgment that a story is novel does not imply that it lies farther from other stories in that collection.

Stories in the AI conditions also became more similar to the suggested AI ideas, consistent with the authors’ anchoring interpretation. 〔Doshi2024 p.5〕 Content similarity alone, however, cannot distinguish psychological anchoring from writers deliberately adopting helpful suggestions. Because the experiment used fixed prompts without open interaction, it also leaves unresolved whether convergence stemmed from the underlying model or from this restricted mode of use. 〔Doshi2024 pp.6–7〕 Providing a creative starting point while narrowing thematic directions offers a plausible account of the results, yet the specific mechanisms require direct testing. Similarly, the scope of short fiction, ordinary participants, and a single task limits inferences about professional creators or lasting changes in writing ability.

This analysis therefore supports reporting average quality, distributional effects across writers, and collection-level diversity alongside one another; aggregating them into a single metric requires stating an explicit objective and weighting scheme. Helping more writers produce competent work and maintaining a diverse body of texts are distinct goals that call for separate judgments. Although the authors’ social dilemma framing highlights how these goals can conflict, the experiment did not measure long-term adoption, reader welfare, or net cultural loss. 〔Doshi2024 p.7〕

This distinction points to testable follow-up designs. Future studies could compare fixed suggestions against a workflow in which writers draft initial ideas independently before receiving AI assistance, measuring individual quality and collection-level diversity simultaneously to test whether quality gains can be retained with less convergence. Subsequent unaided writing tasks could further distinguish immediate output improvements from transferable skill gains. These designs represent proposed tests rather than established findings. Together, the evidence answers the Introduction’s question: generative AI can improve individual works while narrowing differences among them, so evaluating its contribution requires assessing both individual and collective levels alongside the distribution of benefits for a stated creative goal.

## 版本 N

### 中文

#### 摘要

生成式人工智能为何能改善个人创作，却削弱集体多样性？本文依据Doshi和Hauser（2024）的短篇故事实验作批判性分析，不报告新实验。该研究发现，获得AI构思的机会提高了作品的新颖性与有用性评分，收益尤其集中于发散联想任务得分较低的作者；与此同时，同一实验条件内的故事更为相似。〔Doshi2024 pp.3–5〕本文认为，这两种结果分别涉及单篇作品的表现与作品之间的关系，并不矛盾。AI提供的起点可能帮助个人改善作品，也可能使创作趋向相近方向；具体心理机制仍未得到单独识别。因此，评价AI的贡献应同时考察质量、收益分布和作品集合的多样性。该实验支持这一评价区分，但不能据此断言长期文化多样性必然下降。

#### 引言

如果创作工具让每位作者的作品获得更高评价，是否就意味着整个创作领域更有创造力？这一推论忽略了作品之间的关系：平均质量可以提高，可供读者选择的内容差异却未必扩大。本文追问，生成式AI如何使个人创作改善与集体多样性下降并存，这一区分又如何改变贡献评价。核心论点是，单篇作品的评分不足以代表整个作品集合的创造价值。

Doshi和Hauser将作者随机分配至独立写作、可获取一个AI构思或最多五个构思的条件。最终分析包含293篇八句故事，由600名独立评估者评价；主要评分发生在披露AI使用情况之前。〔Doshi2024 p.2〕随机化识别的是获得建议机会的效果，不能把组间差异理解为每多使用一个构思的收益。作者仍负责写作，因此该设计适合研究人机互补如何改变产出。本文以此为依据分析评价层次，不将作品评分的改善等同于作者获得了持久的创作能力。

#### 相关研究与定位

以下定位依据该论文对既有工作的介绍，其引用文献并未另行审阅。作者将研究接续于生成式AI改善工作质量、生产率和人机协作叙事的研究，转而考察兼具经济与表达价值的创造性产出。他们同时区别于人类与AI谁更强的竞赛框架，关注普通参与者借助AI能完成什么。〔Doshi2024 pp.1–2〕〔Doshi2024 pp.5–6〕这一定位使评价对象从机器的独立能力转向协作条件下的人类作品，也要求区分平均收益与收益由谁获得。

概念上，论文沿用新颖性和有用性两个维度，并将其调整为故事评价：前者包含新颖、原创和罕见，后者涉及受众适配、扩展成书的可行性及出版潜力。〔Doshi2024 p.1〕因此，“有用性”是针对特定用途的读者判断，不能直接视为实际出版成绩。作者提出AI既可提供创作起点，也可通过锚定限制构思，〔Doshi2024 p.1〕并在讨论中以社会困境解释个人获益与集体趋同的张力。〔Doshi2024 p.7〕这为本文提供的关键启发是：对单篇作品的新颖性判断，不能替代对作品集合差异的测量。研究的贡献在于把两个层次放入同一实验考察，而非使其中一个指标成为创造力的完整定义。

#### 讨论

图2及正文显示，相比独立写作组，最多五个构思组的新颖性和有用性评分分别提高8.1%和9.0%。图3显示，收益主要集中于发散联想任务得分较低的作者，较高得分者未表现出同样提升。〔Doshi2024 pp.3–4〕这更支持改善起点较低者的作品表现，而非证明AI突破了最高创作水平。任务得分也不应被当作作者全部创作潜能的定论。由此，评价平均效果时应保留受益者差异，否则会掩盖AI在缩小表现差距上的作用。

集体层面的证据使用另一把尺子。研究计算每篇故事的文本嵌入与同组其余故事平均嵌入之间的余弦相似度，再乘以100；一个构思组与最多五个构思组相对基线分别增加0.871和0.718点，图4呈现了这一趋同。〔Doshi2024 pp.4–5〕这些数值不是多样性损失的百分比，也不支持建议越多、同质化就必然越强。个人评分上升与这一结果可以同时成立：作者相对于独立写作的表现有所改善，却共同向相近的内容方向移动。评估者觉得某篇故事新颖，不意味着它与同组其他故事距离更远。

故事也更接近提供的AI构思，这与作者的锚定解释相符。〔Doshi2024 p.5〕然而，内容接近本身不能区分心理锚定与作者主动采纳有帮助的建议；实验采用固定提示且不允许自由互动，也无法单独判断趋同源于模型还是这种使用方式。〔Doshi2024 pp.6–7〕因此，创作支持与方向收敛是解释结果的合理路径，其具体机制仍需检验。短篇写作、普通参与者和一次性任务的边界，同样限制了对专业创作及长期能力变化的推断。

据此，本文主张将平均质量、不同作者的收益和集合多样性并列报告；若要汇总为一个评价，必须说明用途与权重。帮助更多人写出合格作品，与让一个作品库提供更多不同内容，是需要分别判断的目标。作者提出的社会困境揭示了二者可能冲突，但实验未测量长期采用、读者福利或文化损失的净值。〔Doshi2024 p.7〕

这一判断导出可检验的研究方向：比较固定建议与先独立构思、再获得AI帮助的流程，同时测量质量和集合差异，以检验收益能否保留而趋同有所减弱；再通过后续无辅助任务区分即时产出改善与能力迁移。这些是本文提出的检验，而非现有结果。对引言问题的回答由此明确：AI可以改善个人作品，同时收窄作品之间的差异；评价其贡献时，必须同时考察这两个层次及受益者分布，再据具体创作目标作出判断。

### English

#### Abstract

How can generative AI improve individual creative outputs while reducing collective diversity? This article critically analyzes Doshi and Hauser’s (2024) short story experiment; it reports no new experiment. They find that access to AI ideas increases ratings of novelty and usefulness, particularly for writers scoring lower on a divergent association task, while stories within the AI conditions become more similar. 〔Doshi2024 pp.3–5〕 This article argues that these findings concern different properties: the performance of individual works and the relationships among works. AI suggestions may help writers improve their stories while directing them toward similar content, although the specific psychological mechanism remains unresolved. Evaluating AI’s contribution therefore requires attention to quality, the distribution of benefits, and diversity across a collection. The experiment supports this distinction but does not establish an inevitable decline in cultural diversity over time.

#### Introduction

If a creative tool helps every writer receive better evaluations, does the creative field necessarily become more creative? That inference leaves out relationships among works. Average quality can rise without increasing the range of different content available to readers. This article asks how individual improvement and reduced collective diversity can coexist, and how that distinction changes the evaluation of generative AI. Its central argument is that ratings of individual works cannot adequately represent the creative value of an entire collection.

Doshi and Hauser randomly assigned writers to work independently, access one AI idea, or access up to five ideas. Their final analysis included 293 eight-sentence stories evaluated by 600 separate participants, with the primary assessments completed before any disclosure of AI use. 〔Doshi2024 p.2〕 Randomization identifies the effect of access to suggestions; the differences between conditions cannot be interpreted as the benefit of each additional idea actually used. Writers remained responsible for composing their stories, making the design suited to examining how human–AI complementarity changes output. This article uses that evidence to distinguish levels of evaluation while keeping improved output separate from any claim that writers acquired lasting creative abilities.

#### Related work and positioning

The positioning developed here follows the paper’s account of previous research; the publications it cites have not been independently examined. Doshi and Hauser connect their study to work on production quality, productivity, and collaborative storytelling with AI. They extend that framing to creative output with both economic and expressive value. They also distinguish their approach from comparisons that ask whether humans or AI perform better, focusing instead on what ordinary participants can produce with assistance. 〔Doshi2024 pp.1–2〕 〔Doshi2024 pp.5–6〕 This shifts evaluation from the machine’s independent capabilities to human work under conditions of collaboration, making both average gains and their distribution consequential.

Conceptually, the paper adopts novelty and usefulness as dimensions of creativity and adapts them to stories. Novelty combines judgments of newness, originality, and rarity. Usefulness concerns suitability for the intended audience, feasibility of development into a book, and publishing potential. 〔Doshi2024 p.1〕 Usefulness therefore represents readers’ judgments about a particular purpose; it is not observed publication success. The authors propose that AI can provide starting points for creativity while anchoring writers’ ideas, and later invoke a social dilemma to interpret the tension between individual benefit and collective convergence. 〔Doshi2024 p.1〕 〔Doshi2024 p.7〕 For the present question, the consequential insight is that judgments of novelty within individual works cannot substitute for measuring differences across a collection. The paper brings these levels into one experiment without making either measure an exhaustive definition of creativity.

#### Discussion

Figure 2 and the accompanying text show that access to up to five ideas increased novelty ratings by 8.1% and usefulness ratings by 9.0% relative to independent writing. Figure 3 indicates that gains were concentrated among writers with lower divergent association task scores; higher scoring writers did not show comparable improvements. 〔Doshi2024 pp.3–4〕 This pattern better supports improving performance from a lower starting point than demonstrating an expansion of the highest level of creative achievement. Nor should a task score settle a writer’s entire creative potential. Evaluations of average effects should therefore retain differences among beneficiaries, since an average alone can conceal AI’s role in narrowing performance gaps.

Collective diversity was assessed with a different measure. The researchers calculated cosine similarity between each story’s text embedding and the average embedding of the other stories in its condition, then multiplied the result by 100. Relative to the baseline, similarity increased by 0.871 points with access to one idea and by 0.718 points with access to up to five; Figure 4 displays this convergence. 〔Doshi2024 pp.4–5〕 These numbers are not percentages of diversity lost, and they do not support a necessary progression from more suggestions to greater homogenization. Individual improvement and convergence can coexist because a writer can perform better than under independent writing while moving toward content also approached by others. A reader’s judgment that a story is novel does not imply that it lies farther from other stories in the same condition.

Stories also became more similar to the AI ideas provided, consistent with the authors’ anchoring interpretation. 〔Doshi2024 p.5〕 Content similarity alone, however, does not distinguish psychological anchoring from writers deliberately adopting useful suggestions. Fixed prompts and the absence of open interaction also leave unresolved whether convergence arose from the model or this particular way of using it. 〔Doshi2024 pp.6–7〕 Creative support and convergence thus offer a plausible account of the findings, while their specific mechanisms require further testing. The boundaries of short fiction, ordinary participants, and a single task also restrict conclusions about professional creation and lasting changes in ability.

This article consequently proposes reporting average quality, benefits for different writers, and diversity across outputs alongside one another. Combining them into one assessment requires an explicit purpose and weighting. Helping more people produce adequate work and providing a collection with a wider range of content are objectives that require separate judgments. The authors’ social dilemma interpretation identifies a possible conflict between them, but the experiment did not measure long term adoption, reader welfare, or the net value of cultural losses. 〔Doshi2024 p.7〕

This assessment motivates specific tests. Researchers could compare the fixed suggestion procedure with a workflow in which writers first develop ideas independently and then receive AI assistance, measuring quality and collective differences together. That comparison would test whether gains can be retained while convergence is reduced. Subsequent unaided tasks could distinguish immediate output improvement from transferable ability. These are proposed investigations, not established findings. The answer to the Introduction’s question is therefore that AI can improve individual works while narrowing differences among them. Its contribution must be assessed at both levels, together with the distribution of benefits, before it can be judged against a particular creative objective.

## 请反馈

1. 整体偏好：M / N / 持平 / 各有优点。
2. 中文与英文是否有不同偏好；未读的语言请注明。
3. 最需要保留或修改的段落，以及是否发现术语、论证或限定问题。
4. 实际阅读用时、实际修改用时（未记录可如实说明）。
5. 本次评审身份：普通读者 / 相关领域研究者 / 稿件作者。此处为示范稿，身份按实际情况记录。
