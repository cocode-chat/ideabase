use langchain_rust::{ prompt_args };
use crate::core::chain::RETRIEVER_CHAINS;

/// 执行对话调用
/// 
/// 使用检索增强生成(RAG)技术，根据用户输入的消息和指定的集合名称生成响应
/// 
/// # 参数
/// * `message`: 用户输入的消息内容
/// * `collection_name`: 要查询的向量集合名称
/// 
/// # 返回值
/// 返回Result类型，成功时包含生成的响应消息，失败时返回错误信息
pub async fn conversation_call(message: &str, collection_name: &str) -> Result<String, String> {
    // 准备输入变量，将用户消息作为"question"参数
    let input_variables = prompt_args! {
        "question" => message,
    };
    
    // 获取指定集合的检索链
    let chain = RETRIEVER_CHAINS.lock().unwrap().get(collection_name).unwrap().clone();
    
    // 调用检索链生成响应
    let result = chain.invoke(input_variables).await;
    
    // 处理调用结果
    match result {
        Ok(message) => {
            // 返回成功结果
            Ok(message)
        }
        Err(err) => {
            // 记录错误日志并返回错误信息
            log::error!("Error invoking chain: {} {:?}", collection_name, err);
            Err(format!("Error vector search: {:?}", err))
        }
    }
}