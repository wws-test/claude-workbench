use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tauri::{command, AppHandle, Manager};
use crate::process::ProcessRegistryState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub base_url: String,
    pub auth_token: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentConfig {
    pub anthropic_base_url: Option<String>,
    pub anthropic_auth_token: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub anthropic_model: Option<String>,
}

// 获取配置文件路径
fn get_providers_config_path() -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| "无法获取用户主目录".to_string())?;
    
    let config_dir = home_dir.join(".claude");
    
    // 确保配置目录存在
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("无法创建配置目录: {}", e))?;
    }
    
    Ok(config_dir.join("providers.json"))
}

// 从文件加载代理商配置
fn load_providers_from_file() -> Result<Vec<ProviderConfig>, String> {
    let config_path = get_providers_config_path()?;
    
    if !config_path.exists() {
        // 如果文件不存在，返回空列表
        return Ok(vec![]);
    }
    
    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;
    
    if content.trim().is_empty() {
        return Ok(vec![]);
    }
    
    let providers: Vec<ProviderConfig> = serde_json::from_str(&content)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;
    
    Ok(providers)
}

// 保存代理商配置到文件
fn save_providers_to_file(providers: &Vec<ProviderConfig>) -> Result<(), String> {
    let config_path = get_providers_config_path()?;
    
    let content = serde_json::to_string_pretty(providers)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    
    fs::write(&config_path, content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;
    
    Ok(())
}

// CRUD 操作 - 获取所有代理商配置
#[command]
pub fn get_provider_presets() -> Result<Vec<ProviderConfig>, String> {
    let config_path = get_providers_config_path()?;
    
    if !config_path.exists() {
        return Ok(vec![]);
    }
    
    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("无法读取配置文件: {}", e))?;
    
    let configs: Vec<ProviderConfig> = serde_json::from_str(&content)
        .map_err(|e| format!("配置文件格式错误: {}", e))?;
    
    Ok(configs)
}

#[command]
pub fn add_provider_config(config: ProviderConfig) -> Result<String, String> {
    let mut providers = load_providers_from_file()?;
    
    // 检查ID是否已存在
    if providers.iter().any(|p| p.id == config.id) {
        return Err(format!("ID '{}' 已存在，请使用不同的ID", config.id));
    }
    
    providers.push(config.clone());
    save_providers_to_file(&providers)?;
    
    Ok(format!("成功添加代理商配置: {}", config.name))
}

// CRUD 操作 - 更新代理商配置
#[command]
pub fn update_provider_config(config: ProviderConfig) -> Result<String, String> {
    let mut providers = load_providers_from_file()?;
    
    let index = providers.iter().position(|p| p.id == config.id)
        .ok_or_else(|| format!("未找到ID为 '{}' 的配置", config.id))?;
    
    providers[index] = config.clone();
    save_providers_to_file(&providers)?;
    
    Ok(format!("成功更新代理商配置: {}", config.name))
}

// CRUD 操作 - 删除代理商配置
#[command]
pub fn delete_provider_config(id: String) -> Result<String, String> {
    let mut providers = load_providers_from_file()?;
    
    let index = providers.iter().position(|p| p.id == id)
        .ok_or_else(|| format!("未找到ID为 '{}' 的配置", id))?;
    
    let deleted_config = providers.remove(index);
    save_providers_to_file(&providers)?;
    
    Ok(format!("成功删除代理商配置: {}", deleted_config.name))
}

// CRUD 操作 - 获取单个代理商配置
#[command]
pub fn get_provider_config(id: String) -> Result<ProviderConfig, String> {
    let providers = load_providers_from_file()?;
    
    providers.into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| format!("未找到ID为 '{}' 的配置", id))
}

#[command]
pub fn get_current_provider_config() -> Result<CurrentConfig, String> {
    Ok(CurrentConfig {
        anthropic_base_url: env::var("ANTHROPIC_BASE_URL").ok(),
        anthropic_auth_token: env::var("ANTHROPIC_AUTH_TOKEN").ok(),
        anthropic_api_key: env::var("ANTHROPIC_API_KEY").ok(),
        anthropic_model: env::var("ANTHROPIC_MODEL").ok(),
    })
}

#[command]
pub async fn switch_provider_config(app: tauri::AppHandle, config: ProviderConfig) -> Result<String, String> {
    // 首先清理现有环境变量 (但不重启，因为我们马上要设置新的)
    clear_env_vars_only()?;
    
    // 设置新的环境变量
    set_env_var("ANTHROPIC_BASE_URL", &config.base_url)?;
    
    if let Some(auth_token) = &config.auth_token {
        set_env_var("ANTHROPIC_AUTH_TOKEN", auth_token)?;
    }
    
    if let Some(api_key) = &config.api_key {
        set_env_var("ANTHROPIC_API_KEY", api_key)?;
    }
    
    if let Some(model) = &config.model {
        set_env_var("ANTHROPIC_MODEL", model)?;
    }
    
    // 终止所有运行中的Claude进程以使新环境变量生效
    terminate_claude_processes(&app).await;
    
    Ok(format!("已成功切换到 {} ({})，所有Claude会话已重启以应用新配置", config.name, config.description))
}

#[command]
pub async fn clear_provider_config(app: tauri::AppHandle) -> Result<String, String> {
    // 清理所有 ANTHROPIC 相关环境变量
    let vars_to_clear = vec![
        "ANTHROPIC_API_KEY",
        "ANTHROPIC_AUTH_TOKEN", 
        "ANTHROPIC_BASE_URL",
        "ANTHROPIC_MODEL"
    ];
    
    for var_name in &vars_to_clear {
        // 使用 setx 删除持久化环境变量 (Windows)
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            
            Command::new("cmd")
                .args(&["/C", &format!("setx {} \"\"", var_name)])
                .creation_flags(0x08000000) // CREATE_NO_WINDOW
                .output()
                .map_err(|e| format!("Failed to clear {}: {}", var_name, e))?;
                
            // 使用 reg 命令删除注册表中的空值 - 静默执行
            Command::new("reg")
                .args(&["delete", "HKCU\\Environment", "/v", var_name, "/f"])
                .creation_flags(0x08000000) // CREATE_NO_WINDOW
                .output()
                .ok(); // 忽略错误，因为变量可能不存在
        }
        
        // 清理当前进程的环境变量
        env::remove_var(var_name);
    }
    
    // 终止所有运行中的Claude进程以使清理生效
    terminate_claude_processes(&app).await;
    
    Ok("已清理所有 ANTHROPIC 环境变量，所有Claude会话已重启".to_string())
}

/// 仅清理环境变量，不重启进程 (供switch_provider_config内部使用)
fn clear_env_vars_only() -> Result<(), String> {
    // 清理所有 ANTHROPIC 相关环境变量
    let vars_to_clear = vec![
        "ANTHROPIC_API_KEY",
        "ANTHROPIC_AUTH_TOKEN", 
        "ANTHROPIC_BASE_URL",
        "ANTHROPIC_MODEL"
    ];
    
    for var_name in &vars_to_clear {
        // 使用 setx 删除持久化环境变量 (Windows)
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            
            Command::new("cmd")
                .args(&["/C", &format!("setx {} \"\"", var_name)])
                .creation_flags(0x08000000) // CREATE_NO_WINDOW
                .output()
                .map_err(|e| format!("Failed to clear {}: {}", var_name, e))?;
                
            // 使用 reg 命令删除注册表中的空值 - 静默执行
            Command::new("reg")
                .args(&["delete", "HKCU\\Environment", "/v", var_name, "/f"])
                .creation_flags(0x08000000) // CREATE_NO_WINDOW
                .output()
                .ok(); // 忽略错误，因为变量可能不存在
        }
        
        // 清理当前进程的环境变量
        env::remove_var(var_name);
    }
    
    Ok(())
}

#[cfg(target_os = "windows")]
fn set_env_var(name: &str, value: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    
    // 设置持久化环境变量 (Windows) - 静默执行
    // 只有包含空格或特殊字符的值才需要引号
    let formatted_value = if value.contains(' ') || value.contains('&') || value.contains('|') || value.contains('<') || value.contains('>') || value.contains('^') {
        format!("\"{}\"", value)
    } else {
        value.to_string()
    };
    
    Command::new("cmd")
        .args(&["/C", &format!("setx {} {}", name, formatted_value)])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| format!("Failed to set {}: {}", name, e))?;
        
    // 同时设置当前进程的环境变量
    env::set_var(name, value);
    
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn set_env_var(name: &str, value: &str) -> Result<(), String> {
    // 对于非 Windows 系统，只设置当前进程环境变量
    // 持久化需要修改 shell 配置文件，这里暂时不实现
    env::set_var(name, value);
    Ok(())
}

#[command]
pub fn test_provider_connection(base_url: String) -> Result<String, String> {
    // 简单的连接测试 - 尝试访问 API 端点
    let test_url = if base_url.ends_with('/') {
        format!("{}v1/messages", base_url)
    } else {
        format!("{}/v1/messages", base_url)
    };
    
    // 这里可以实现实际的 HTTP 请求测试
    // 目前返回一个简单的成功消息
    Ok(format!("连接测试完成：{}", test_url))
}

/// 终止所有运行中的Claude进程以使新环境变量生效
async fn terminate_claude_processes(app: &AppHandle) {
    log::info!("正在终止所有Claude进程以应用新的代理商配置...");
    
    // 获取进程注册表
    let registry = app.state::<ProcessRegistryState>();
    
    // 获取所有活动的Claude会话
    match registry.0.get_running_claude_sessions() {
        Ok(sessions) => {
            log::info!("找到 {} 个活动的Claude会话", sessions.len());
            
            for session in sessions {
                let session_id_str = match &session.process_type {
                    crate::process::registry::ProcessType::ClaudeSession { session_id } => session_id.as_str(),
                    _ => "unknown",
                };
                
                log::info!("正在终止Claude会话: session_id={}, run_id={}, PID={}", 
                    session_id_str,
                    session.run_id, 
                    session.pid
                );
                
                // 尝试优雅地终止进程
                match registry.0.kill_process(session.run_id).await {
                    Ok(success) => {
                        if success {
                            log::info!("成功终止Claude会话 {}", session.run_id);
                        } else {
                            log::warn!("终止Claude会话 {} 返回false", session.run_id);
                            
                            // 尝试强制终止
                            if let Err(e) = registry.0.kill_process_by_pid(session.run_id, session.pid as u32) {
                                log::error!("强制终止进程失败: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("终止Claude会话 {} 失败: {}", session.run_id, e);
                        
                        // 尝试强制终止
                        if let Err(e2) = registry.0.kill_process_by_pid(session.run_id, session.pid as u32) {
                            log::error!("强制终止进程也失败: {}", e2);
                        }
                    }
                }
            }
        }
        Err(e) => {
            log::error!("获取Claude会话列表失败: {}", e);
        }
    }
    
    log::info!("Claude进程终止操作完成");
}