use langchain_rust::{
    embedding::openai::OpenAiEmbedder,
    llm::{OpenAIConfig, OpenAI},
};

/// 获取OpenAI文本嵌入器(Embedder)
/// 
/// 从环境变量读取配置并初始化OpenAI文本嵌入器，用于将文本转换为向量表示
/// 
/// # 返回值
/// 返回Result类型，成功时包含OpenAiEmbedder实例，失败时返回错误信息
pub fn get_embedder_llm() -> Result<OpenAiEmbedder<OpenAIConfig>, String> {
    // 从环境变量获取嵌入器基础URL
    let embedder_base_url = std::env::var("EMBEDDING_BASE_URL")
        .map_err(|_| "Missing EMBEDDING_BASE_URL environment variable")?;
    
    // 从环境变量获取嵌入器模型名称
    let embedder_model = std::env::var("EMBEDDING_MODEL")
        .map_err(|_| "Missing EMBEDDING_MODEL environment variable")?;
    
    // 从环境变量获取嵌入器API密钥
    let embedder_api_key = std::env::var("EMBEDDING_API_KEY")
        .map_err(|_| "Missing EMBEDDING_API_KEY environment variable")?;

    // 初始化OpenAI嵌入器配置
    let embedder = OpenAiEmbedder::new(
        OpenAIConfig::new()
            .with_api_base(embedder_base_url)  // 设置API基础URL
            .with_api_key(embedder_api_key)     // 设置API密钥
    ).with_model(embedder_model);  // 设置模型名称
    
    // 返回初始化好的嵌入器
    Ok(embedder)
}

/// 获取OpenAI对话模型
/// 
/// 从环境变量读取配置并初始化OpenAI对话模型，用于生成自然语言响应
/// 
/// # 返回值
/// 返回Result类型，成功时包含OpenAI实例，失败时返回错误信息
pub fn get_conversation_llm() -> Result<OpenAI<OpenAIConfig>, String> {
    // 从环境变量获取对话模型基础URL
    let conversation_base_url = std::env::var("CONVERSATION_BASE_URL")
        .map_err(|_| "Missing CONVERSATION_BASE_URL environment variable")?;
    
    // 从环境变量获取对话模型名称
    let conversation_model = std::env::var("CONVERSATION_MODEL")
        .map_err(|_| "Missing CONVERSATION_MODEL environment variable")?;
    
    // 从环境变量获取对话模型API密钥
    let conversation_api_key = std::env::var("CONVERSATION_API_KEY")
        .map_err(|_| "Missing CONVERSATION_API_KEY environment variable")?;

    // 初始化OpenAI对话模型配置
    let conversation = OpenAI::new(
        OpenAIConfig::new()
            .with_api_base(conversation_base_url)  // 设置API基础URL
            .with_api_key(conversation_api_key)     // 设置API密钥
    ).with_model(conversation_model);  // 设置模型名称
    
    // 返回初始化好的对话模型
    Ok(conversation)
}