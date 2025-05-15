![Ideabase](.doc/logo.jpg)

[Ideabase](https://github.com/cocode-chat/ideabase) - AI时代的企业级智能体基础设施

Ideabase作为AI时代的Firebase替代方案，为企业智能体提供强大的数据支撑平台。我们致力于让业务数据"自然生长"出智能，帮助企业从传统的"数据拥有"模式向"数据智能"模式转型升级。

基于企业级开源工具和Rust编程语言构建，Ideabase不仅提供类似Firebase的核心功能，更在以下方面实现突破：
- 企业级安全防护机制
- 极致性能优化
- 原生AI能力集成

通过简单的配置，您的数据库即可升级为智能体平台：
- 支持AI驱动的自然语言交互
- 赋能个性化智能客服系统开发
- 构建以企业数据为核心的业务智能体

- [x] Hosted MySQL Database.
  - [x] REST
  - [ ] Cache
  - [ ] Realtime subscriptions
- [ ] AI
  - [x] RAG base on hosted database
  - [ ] RAG base on upload file
  - [ ] MCP autogen
- [ ] File Storage
- [ ] Authentication
- [ ] Authorization
- [ ] Dashboard

# Install
注意替换 [LLM api key](.run/Docker-run-env.properties) 中的EMBEDDING_API_KEY, CONVERSATION_API_KEY
```shell
git clone git@github.com:cocode-chat/ideabase.git
cd ideabase
sh .run/Docker-compose.sh
```

# MySQL DB RESTful API 
See [RESTful](.doc/README-restful.md) docs.

这里特别感谢腾讯开源的[APIJSON](http://apijson.cn/)项目，我们高度认可其协议设计并保持兼容。

与APIJSON采用的深度优先遍历节点不同，Ideabase创新性地实现了广度优先遍历节点合并子查询技术，这一优化显著减少了数据库IO操作。虽然当前实现代码在美观度上还有提升空间，但性能优势已经得到验证。

尤为重要的是，我们成功实现了跨库关联查询功能，这一特性对于资源受限的项目具有极高的实用价值，能够大幅提升开发效率。

# RAG base on hosted database
See [RAG](.doc/README-rag.md) docs.