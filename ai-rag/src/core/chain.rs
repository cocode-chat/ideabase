use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use langchain_rust::{
    chain::{Chain, ConversationalRetrieverChainBuilder},
    memory::SimpleMemory,
    prompt::HumanMessagePromptTemplate, schemas::Message,
    vectorstore::Retriever,
    fmt_message, fmt_template, message_formatter, template_jinja2
};

use crate::core::llm::get_conversation_llm;
use crate::core::vector::build_vector_store;

/// 全局检索链存储
/// 
/// 使用Lazy和Mutex包装的HashMap，用于存储不同集合名称对应的检索链
pub static RETRIEVER_CHAINS: Lazy<Mutex<HashMap<String, Arc<dyn Chain>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

/// 构建检索增强生成(RAG)链
/// 
/// 为指定的集合名称构建一个完整的检索增强生成链，
/// 包含LLM对话模型、提示模板和向量存储检索器
/// 
/// # 参数
/// * `collection_name`: 要构建检索链的集合名称
/// 
/// # 返回值
/// 返回Result类型，成功时包含构建好的检索链，失败时返回错误信息
pub async fn build_retriever_chain(collection_name: &str) -> Result<Arc<dyn Chain>, String> {
    // 获取LLM对话模型
    let llm_conversation = get_conversation_llm()?;
    
    // 构建提示模板
    let prompt_template = message_formatter![
        // 系统消息：定义AI角色
        fmt_message!(Message::new_system_message("你是一个电商客服助手！")),
        // 用户消息模板：包含上下文和问题的Jinja2模板
        fmt_template!(HumanMessagePromptTemplate::new(template_jinja2!(
            "使用以下上下文来回答最后的问题。如果你不知道答案，就说你不知道，不要试图编造答案。
            {{context}}

            问题:{{question}}
            有用的答案: ", "context", "question")))
    ];
    
    // 构建向量存储
    let store = build_vector_store(collection_name).await?;
    
    // 构建完整的对话检索链
    let conversation_chain = ConversationalRetrieverChainBuilder::new()
        .llm(llm_conversation)  // 设置LLM模型
        .rephrase_question(true)  // 启用问题重述
        .memory(SimpleMemory::new().into())  // 添加简单内存
        .retriever(Retriever::new(store, 5))  // 设置检索器，返回5个最相关文档
        .prompt(prompt_template)  // 设置提示模板
        .build()
        .map_err(|e| format!("Error building ConversationalChain: {:?}", e))?;
    
    // 返回构建好的检索链
    Ok(Arc::new(conversation_chain))
}