# Ideabase
[Ideabase](https://ideabase.chat) 是AI时代的Firebase。

我们正在使用企业级开源工具和Rust编程语言构建Ideabase的功能，它提供了类似Firebase的能力，但具有更强大的安全性和顶级性能。

我们还可以通过简单的配置使您的数据库具有AI知识库功能，使其能够支持AI驱动的对话以满足您的业务需求，基于此可以研发适用于个性化智能客服智能体和以企业业务数据为中心的智能代理。

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
注意替换 [LLM api key](.run/Docker-run-env.properties) 中的EMBEDDING_API_KEY,CONVERSATION_API_KEY
```shell
git clone git@github.com:cocode-chat/ideabase.git
cd ideabase
sh .run/Docker-compose.sh
```

# MySQL DB Restful API 
See [Restful](README-restful.md) docs.

这里需要感谢腾讯开源的[APIJSON](http://apijson.cn/)项目，它的协议设计我们很喜欢并兼容使用。

但 [APIJSON](http://apijson.cn/) 是深度优先遍历节点，会导致过多的数据库IO, Ideabase采用广度优先遍历节点合并子查询，减少数据库IO。当然这也导致目前代码看起来比较丑陋。

# RAG base on hosted database
See [RAG](README-rag.md) docs.