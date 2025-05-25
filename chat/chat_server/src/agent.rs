use ai_sdk::{AiAdapter, AiSdk, OllamaAdapter, OpenAIAdapter};
use core_lib::{
    AdapterType, Agent, AgentContext, AgentDecision, AgentError, AgentType, ChatAgents,
};

pub enum AgentVariant {
    Proxy(ProxyAgent),
    Reply(ReplyAgent),
    Tap(TapAgent),
}

#[allow(unused)]
pub struct ProxyAgent {
    pub name: String,
    pub prompt: String,
    pub args: serde_json::Value,
    pub adapter: AiAdapter,
}

#[allow(unused)]
pub struct ReplyAgent {
    pub name: String,
    pub prompt: String,
    pub args: serde_json::Value,
    pub adapter: AiAdapter,
}

#[allow(unused)]
pub struct TapAgent {
    pub name: String,
    pub prompt: String,
    pub args: serde_json::Value,
    pub adapter: AiAdapter,
}

impl Agent for ProxyAgent {
    async fn process(&self, msg: &str, _ctx: &AgentContext) -> Result<AgentDecision, AgentError> {
        let msg = format!("{} {}", self.prompt, msg);
        let messages = vec![ai_sdk::Message::user(msg.clone())];
        println!("messages: {:?}", messages);
        let response = self.adapter.complete(&messages).await?;
        Ok(AgentDecision::Modify(response))
    }
}
impl Agent for ReplyAgent {
    async fn process(&self, msg: &str, _ctx: &AgentContext) -> Result<AgentDecision, AgentError> {
        //1. create embedding for the message
        //2. search the embedding in the database
        //3. query llm with prompt and related docs as context
        //4. create embedding for the response
        //5. save the embedding and its relation to the message
        let msg = format!("{} {}", self.prompt, msg);
        let messages = vec![ai_sdk::Message::user(msg.clone())];
        let response = self.adapter.complete(&messages).await?;
        Ok(AgentDecision::Reply(response))
    }
}
impl Agent for TapAgent {
    async fn process(&self, _msg: &str, _ctx: &AgentContext) -> Result<AgentDecision, AgentError> {
        Ok(AgentDecision::None)
    }
}

impl Agent for AgentVariant {
    async fn process(&self, msg: &str, ctx: &AgentContext) -> Result<AgentDecision, AgentError> {
        match self {
            AgentVariant::Proxy(agent) => agent.process(msg, ctx).await,
            AgentVariant::Reply(agent) => agent.process(msg, ctx).await,
            AgentVariant::Tap(agent) => agent.process(msg, ctx).await,
        }
    }
}

impl From<ChatAgents> for AgentVariant {
    fn from(agent: ChatAgents) -> Self {
        let adapter = match agent.adapter {
            AdapterType::Openai => AiAdapter::OpenAI(OpenAIAdapter::default()),
            AdapterType::Ollama => AiAdapter::Ollama(OllamaAdapter::default()),
        };
        match agent.r#type {
            AgentType::Proxy => AgentVariant::Proxy(ProxyAgent {
                name: agent.name,
                prompt: agent.prompt,
                args: agent.args,
                adapter,
            }),
            AgentType::Reply => AgentVariant::Reply(ReplyAgent {
                name: agent.name,
                prompt: agent.prompt,
                args: agent.args,
                adapter,
            }),
            AgentType::Tap => AgentVariant::Tap(TapAgent {
                name: agent.name,
                prompt: agent.prompt,
                args: agent.args,
                adapter,
            }),
        }
    }
}

impl From<ProxyAgent> for AgentVariant {
    fn from(agent: ProxyAgent) -> Self {
        AgentVariant::Proxy(agent)
    }
}

impl From<ReplyAgent> for AgentVariant {
    fn from(agent: ReplyAgent) -> Self {
        AgentVariant::Reply(agent)
    }
}

impl From<TapAgent> for AgentVariant {
    fn from(agent: TapAgent) -> Self {
        AgentVariant::Tap(agent)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::AppState;

    use super::*;
    use anyhow::Result;
    #[ignore]
    #[tokio::test]
    async fn test_proxy_agent() {
        let agent=ChatAgents {
            id: 1,
            name: "proxy".to_string(),
            r#type: AgentType::Proxy,
            prompt: "You're the world's best translator,You understand English and Chinese well, also their culture and history. If the original text is English, you will translate it to elegant, authentic Simplified Chinese. If the original text is Chinese, you will translate it to elegant, authentic English. Only return the translated sentences, no other text or comments. Belows are the text to translate:".to_string(),
            args: serde_json::Value::Null,
            adapter: AdapterType::Openai,
            model: "gpt-4o".to_string(),
            chat_id: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let agent_variant = AgentVariant::from(agent);
        let decision = agent_variant
            .process(
                "你好，世界！",
                &AgentContext {
                    chat_id: 1,
                    user_id: 1,
                    workspace_id: 1,
                },
            )
            .await
            .unwrap();
        match decision {
            AgentDecision::Modify(content) => {
                assert_eq!(content.to_lowercase(), "hello, world!".to_string());
            }
            _ => {
                panic!("decision is not Modify");
            }
        }
    }
    #[ignore]
    #[tokio::test]
    async fn agent_variant_test() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let agents = state.list_agents(2, 1).await?;
        let agent = agents.iter().find(|a| a.name == "translation2").unwrap();
        // println!("agent: {:?}", agent.adapter);
        let agent_variant = AgentVariant::from(agent.clone());
        let decision = agent_variant
            .process(
                "你好，世界！",
                &AgentContext {
                    chat_id: 2,
                    user_id: 1,
                    workspace_id: 1,
                },
            )
            .await
            .unwrap();
        match decision {
            AgentDecision::Modify(content) => {
                assert_eq!(content.to_lowercase(), "hello, world!".to_string());
            }
            _ => {
                panic!("decision is not Modify");
            }
        }
        Ok(())
    }
}
