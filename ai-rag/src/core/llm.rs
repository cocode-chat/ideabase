use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use langchain_rust::{
    chain::{Chain, ConversationalRetrieverChainBuilder},
    embedding::openai::OpenAiEmbedder,
    llm::{OpenAIConfig, OpenAI},
    memory::SimpleMemory,
    prompt::HumanMessagePromptTemplate, schemas::Message,
    vectorstore::Retriever,
    fmt_message, fmt_template, message_formatter, template_jinja2
};

use once_cell::sync::Lazy;
use crate::core::vector::build_vector_store;

pub static RETRIEVER_CHAINS: Lazy<Mutex<HashMap<String, Arc<dyn Chain>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub async fn build_retriever_chain(collection_name: &str) -> Result<Arc<dyn Chain>, String> {
    let llm_conversation = get_llm_conversation()?;
    let prompt_template = message_formatter![
        fmt_message!(Message::new_system_message("你是一个电商客服助手！")),
        fmt_template!(HumanMessagePromptTemplate::new(template_jinja2!("
            使用以下上下文来回答最后的问题。如果你不知道答案，就说你不知道，不要试图编造答案。
            {{context}}

            问题:{{question}}
            有用的答案: ",  "context", "question")))
    ];
    let store = build_vector_store(collection_name).await?;
    let conversation_chain = ConversationalRetrieverChainBuilder::new()
        .llm(llm_conversation)
        .rephrase_question(true)
        .memory(SimpleMemory::new().into())
        .retriever(Retriever::new(store, 5))
        .prompt(prompt_template)
        .build()
        .map_err(|e| format!("Error building ConversationalChain: {:?}", e))?;
    Ok(Arc::new(conversation_chain))
}

pub fn get_llm_embedder() -> Result<OpenAiEmbedder<OpenAIConfig>, String> {
    // 从环境变量获取嵌入器(Embedder)配置
    let embedder_base_url = std::env::var("EMBEDDING_BASE_URL")
        .map_err(|_| "Missing EMBEDDING_BASE_URL environment variable")?;
    let embedder_model = std::env::var("EMBEDDING_MODEL")
        .map_err(|_| "Missing EMBEDDING_MODEL environment variable")?;
    let embedder_api_key = std::env::var("EMBEDDING_API_KEY")
        .map_err(|_| "Missing EMBEDDING_API_KEY environment variable")?;

    // 初始化OpenAI嵌入器(Embedder), 用于将文本转换为向量表示
    let embedder = OpenAiEmbedder::new(OpenAIConfig::new()
            .with_api_base(embedder_base_url)
            .with_api_key(embedder_api_key)
    ).with_model(embedder_model);
    Ok(embedder)
}


pub fn get_llm_conversation() -> Result<OpenAI<OpenAIConfig>, String> {
    // 从环境变量获取对话模型(Conversation)配置
    let conversation_base_url = std::env::var("CONVERSATION_BASE_URL")
        .map_err(|_| "Missing CONVERSATION_BASE_URL environment variable")?;
    let conversation_model = std::env::var("CONVERSATION_MODEL")
        .map_err(|_| "Missing CONVERSATION_MODEL environment variable")?;
    let conversation_api_key = std::env::var("CONVERSATION_API_KEY")
        .map_err(|_| "Missing CONVERSATION_API_KEY environment variable")?;

    // 初始化OpenAI对话模型, 用于生成自然语言响应
    let conversation = OpenAI::new(OpenAIConfig::new()
            .with_api_base(conversation_base_url)
            .with_api_key(conversation_api_key)
        ).with_model(conversation_model);
    Ok(conversation)
}