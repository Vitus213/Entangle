use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;

// ===== 共享类型 =====

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
    nickname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    phone: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AuthResponse {
    token: String,
    user: UserResponse,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct UserResponse {
    id: String,
    email: String,
    nickname: String,
    avatar_url: Option<String>,
    role: Option<String>,
    email_verified: bool,
    created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Document {
    id: String,
    title: String,
    content: String,
    is_public: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    folder_id: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CreateDocumentRequest {
    title: String,
    content: String,
    is_public: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    folder_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct UpdateDocumentRequest {
    title: String,
    content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Folder {
    id: String,
    name: String,
    parent_id: Option<String>,
    owner_id: String,
    created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct FolderTree {
    id: String,
    name: String,
    parent_id: Option<String>,
    children: Vec<FolderTree>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CreateFolderRequest {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Tag {
    id: String,
    name: String,
    color: String,
    owner_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TagWithCount {
    tag: Tag,
    document_count: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CreateTagRequest {
    name: String,
    color: String,
}

// ===== LocalStorage 辅助函数 =====

const API_BASE: &str = "http://127.0.0.1:3000";

fn get_token() -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item("token").ok()?
}

fn save_token(token: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("token", token);
        }
    }
}

fn clear_token() {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("token");
        }
    }
}

// ===== API 客户端 =====

async fn register_api(email: String, password: String, nickname: String) -> Result<AuthResponse, String> {
    let response = Request::post(&format!("{}/api/auth/register", API_BASE))
        .json(&RegisterRequest { email, password, nickname, phone: None })
        .map_err(|e| format!("请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(format!("注册失败 ({}): {}", status, body))
    }
}

async fn login_api(email: String, password: String) -> Result<AuthResponse, String> {
    let response = Request::post(&format!("{}/api/auth/login", API_BASE))
        .json(&LoginRequest { email, password })
        .map_err(|e| format!("请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("登录失败: {}", response.status()))
    }
}

async fn fetch_documents(token: &str) -> Result<Vec<Document>, String> {
    let response = Request::get(&format!("{}/api/documents/my", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("获取文档失败: {}", response.status()))
    }
}

async fn create_document_api(token: &str, title: String) -> Result<Document, String> {
    let response = Request::post(&format!("{}/api/documents", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&CreateDocumentRequest {
            title,
            content: String::new(),
            is_public: false,
            folder_id: None,
        })
        .map_err(|e| format!("请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("创建失败: {}", response.status()))
    }
}

async fn fetch_document(token: &str, id: &str) -> Result<Document, String> {
    let response = Request::get(&format!("{}/api/documents/{}", API_BASE, id))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("获取文档失败: {}", response.status()))
    }
}

async fn update_document_api(token: &str, id: &str, title: String, content: String) -> Result<Document, String> {
    let response = Request::put(&format!("{}/api/documents/{}", API_BASE, id))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&UpdateDocumentRequest { title, content })
        .map_err(|e| format!("请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("更新失败: {}", response.status()))
    }
}

// 文件夹相关 API
async fn fetch_folder_tree(token: &str) -> Result<Vec<FolderTree>, String> {
    let response = Request::get(&format!("{}/api/folders/tree", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("获取文件夹失败: {}", response.status()))
    }
}

async fn create_folder_api(token: &str, name: String, parent_id: Option<String>) -> Result<Folder, String> {
    let response = Request::post(&format!("{}/api/folders", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&CreateFolderRequest { name, parent_id })
        .map_err(|e| format!("请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("创建文件夹失败: {}", response.status()))
    }
}

// 标签相关 API
async fn fetch_tags(token: &str) -> Result<Vec<TagWithCount>, String> {
    let response = Request::get(&format!("{}/api/tags", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("获取标签失败: {}", response.status()))
    }
}

async fn create_tag_api(token: &str, name: String, color: String) -> Result<TagWithCount, String> {
    let response = Request::post(&format!("{}/api/tags", API_BASE))
        .header("Authorization", &format!("Bearer {}", token))
        .json(&CreateTagRequest { name, color })
        .map_err(|e| format!("请求失败: {}", e))?
        .send()
        .await
        .map_err(|e| format!("网络错误: {}", e))?;

    if response.ok() {
        response.json().await.map_err(|e| format!("解析失败: {}", e))
    } else {
        Err(format!("创建标签失败: {}", response.status()))
    }
}

// ===== 注册页面 =====

#[component]
fn RegisterPage() -> impl IntoView {
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (nickname, set_nickname) = create_signal(String::new());
    let (error, set_error) = create_signal(None::<String>);
    let (loading, set_loading) = create_signal(false);

    let navigate = use_navigate();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set(None);

        let email_val = email.get();
        let password_val = password.get();
        let nickname_val = nickname.get();
        let nav = navigate.clone();

        spawn_local(async move {
            match register_api(email_val, password_val, nickname_val).await {
                Ok(response) => {
                    save_token(&response.token);
                    nav("/documents", Default::default());
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="auth-container">
            <h1>"注册 Entangle"</h1>
            <form on:submit=on_submit>
                <div class="form-group">
                    <label>"昵称"</label>
                    <input
                        type="text"
                        placeholder="你的昵称"
                        prop:value=move || nickname.get()
                        on:input=move |ev| set_nickname.set(event_target_value(&ev))
                        required
                    />
                </div>
                <div class="form-group">
                    <label>"邮箱"</label>
                    <input
                        type="email"
                        placeholder="your@email.com"
                        prop:value=move || email.get()
                        on:input=move |ev| set_email.set(event_target_value(&ev))
                        required
                    />
                </div>
                <div class="form-group">
                    <label>"密码"</label>
                    <input
                        type="password"
                        placeholder="至少6位"
                        prop:value=move || password.get()
                        on:input=move |ev| set_password.set(event_target_value(&ev))
                        required
                        minlength="6"
                    />
                </div>
                {move || error.get().map(|e| view! {
                    <div class="error">{e}</div>
                })}
                <button type="submit" class="btn" disabled=move || loading.get()>
                    {move || if loading.get() { "注册中..." } else { "注册" }}
                </button>
                <p class="auth-link">
                    "已有账号？"
                    <a href="/">"去登录"</a>
                </p>
            </form>
        </div>
    }
}

// ===== 登录页面 =====

#[component]
fn LoginPage() -> impl IntoView {
    let (email, set_email) = create_signal(String::from("demo@example.com"));
    let (password, set_password) = create_signal(String::from("demo123"));
    let (error, set_error) = create_signal(None::<String>);
    let (loading, set_loading) = create_signal(false);

    let navigate = use_navigate();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set(None);

        let email_val = email.get();
        let password_val = password.get();
        let nav = navigate.clone();

        spawn_local(async move {
            match login_api(email_val, password_val).await {
                Ok(response) => {
                    save_token(&response.token);
                    nav("/documents", Default::default());
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="auth-container">
            <h1>"Entangle 登录"</h1>
            <form on:submit=on_submit>
                <div class="form-group">
                    <label>"邮箱"</label>
                    <input
                        type="email"
                        placeholder="your@email.com"
                        prop:value=move || email.get()
                        on:input=move |ev| set_email.set(event_target_value(&ev))
                        required
                    />
                </div>
                <div class="form-group">
                    <label>"密码"</label>
                    <input
                        type="password"
                        placeholder="••••••••"
                        prop:value=move || password.get()
                        on:input=move |ev| set_password.set(event_target_value(&ev))
                        required
                    />
                </div>
                {move || error.get().map(|e| view! {
                    <div class="error">{e}</div>
                })}
                <button type="submit" class="btn" disabled=move || loading.get()>
                    {move || if loading.get() { "登录中..." } else { "登录" }}
                </button>
                <p class="auth-link">
                    "还没有账号？"
                    <a href="/register">"注册"</a>
                </p>
            </form>
        </div>
    }
}

// ===== 文档列表页面 =====

#[component]
fn DocumentsPage() -> impl IntoView {
    let (documents, set_documents) = create_signal(Vec::<Document>::new());
    let (folders, set_folders) = create_signal(Vec::<FolderTree>::new());
    let (tags, set_tags) = create_signal(Vec::<TagWithCount>::new());
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal(None::<String>);

    // 创建文档
    let (show_create_doc, set_show_create_doc) = create_signal(false);
    let (new_doc_title, set_new_doc_title) = create_signal(String::from("新文档"));

    // 创建文件夹
    let (show_create_folder, set_show_create_folder) = create_signal(false);
    let (new_folder_name, set_new_folder_name) = create_signal(String::new());

    // 创建标签
    let (show_create_tag, set_show_create_tag) = create_signal(false);
    let (new_tag_name, set_new_tag_name) = create_signal(String::new());
    let (new_tag_color, set_new_tag_color) = create_signal(String::from("#3B82F6"));

    let (sidebar_collapsed, _set_sidebar_collapsed) = create_signal(false);

    let navigate = use_navigate();
    let nav_clone1 = navigate.clone();

    // 加载所有数据
    create_effect(move |_| {
        if let Some(token) = get_token() {
            set_loading.set(true);

            // 加载文档
            let token_clone = token.clone();
            spawn_local(async move {
                match fetch_documents(&token_clone).await {
                    Ok(docs) => set_documents.set(docs),
                    Err(e) => set_error.set(Some(e)),
                }
            });

            // 加载文件夹
            let token_clone = token.clone();
            spawn_local(async move {
                match fetch_folder_tree(&token_clone).await {
                    Ok(tree) => set_folders.set(tree),
                    Err(e) => leptos::logging::error!("加载文件夹失败: {}", e),
                }
            });

            // 加载标签
            spawn_local(async move {
                match fetch_tags(&token).await {
                    Ok(tag_list) => set_tags.set(tag_list),
                    Err(e) => leptos::logging::error!("加载标签失败: {}", e),
                }
                set_loading.set(false);
            });
        } else {
            nav_clone1("/", Default::default());
        }
    });

    // 创建文档
    let nav_for_create = navigate.clone();
    let create_document = move |_| {
        if let Some(token) = get_token() {
            let title = new_doc_title.get();
            let nav = nav_for_create.clone();
            spawn_local(async move {
                match create_document_api(&token, title).await {
                    Ok(doc) => nav(&format!("/editor/{}", doc.id), Default::default()),
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    // 创建文件夹
    let create_folder = move |_| {
        if let Some(token) = get_token() {
            let name = new_folder_name.get();
            spawn_local(async move {
                match create_folder_api(&token, name, None).await {
                    Ok(_) => {
                        set_show_create_folder.set(false);
                        set_new_folder_name.set(String::new());
                        // 重新加载文件夹树
                        if let Ok(tree) = fetch_folder_tree(&token).await {
                            set_folders.set(tree);
                        }
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    // 创建标签
    let create_tag = move |_| {
        if let Some(token) = get_token() {
            let name = new_tag_name.get();
            let color = new_tag_color.get();
            spawn_local(async move {
                match create_tag_api(&token, name, color).await {
                    Ok(tag_with_count) => {
                        set_show_create_tag.set(false);
                        set_new_tag_name.set(String::new());
                        // 添加到标签列表
                        let mut current_tags = tags.get();
                        current_tags.push(tag_with_count);
                        set_tags.set(current_tags);
                    }
                    Err(e) => set_error.set(Some(e)),
                }
            });
        }
    };

    // 登出
    let nav_for_logout = navigate.clone();
    let logout = move |_| {
        clear_token();
        nav_for_logout("/", Default::default());
    };

    view! {
        <div class="app-container">
            // 顶部导航栏
            <div class="navbar">
                <div class="navbar-brand">
                    <h1>"Entangle"</h1>
                </div>
                <div class="navbar-actions">
                    <button class="btn btn-primary" on:click=move |_| set_show_create_doc.set(!show_create_doc.get())>
                        "+ 新建文档"
                    </button>
                    <button class="btn btn-secondary" on:click=logout>
                        "退出登录"
                    </button>
                </div>
            </div>

            <div class="main-layout">
                // 侧边栏
                <div class="sidebar" class:collapsed=move || sidebar_collapsed.get()>
                    <div class="sidebar-section">
                        <div class="section-header">
                            <h3>"文件夹"</h3>
                            <button class="btn-icon" on:click=move |_| set_show_create_folder.set(!show_create_folder.get())>
                                "+"
                            </button>
                        </div>

                        {move || show_create_folder.get().then(|| view! {
                            <div class="create-form-inline">
                                <input
                                    type="text"
                                    placeholder="文件夹名称"
                                    prop:value=move || new_folder_name.get()
                                    on:input=move |ev| set_new_folder_name.set(event_target_value(&ev))
                                    class="input-sm"
                                />
                                <button class="btn-sm" on:click=create_folder>"创建"</button>
                                <button class="btn-sm btn-cancel" on:click=move |_| set_show_create_folder.set(false)>"取消"</button>
                            </div>
                        })}

                        <div class="folder-list">
                            <For
                                each=move || folders.get()
                                key=|folder| folder.id.clone()
                                children=|folder: FolderTree| {
                                    view! {
                                        <div class="folder-item">
                                            <span class="folder-icon">"📁"</span>
                                            <span class="folder-name">{&folder.name}</span>
                                        </div>
                                    }
                                }
                            />
                        </div>
                    </div>

                    <div class="sidebar-section">
                        <div class="section-header">
                            <h3>"标签"</h3>
                            <button class="btn-icon" on:click=move |_| set_show_create_tag.set(!show_create_tag.get())>
                                "+"
                            </button>
                        </div>

                        {move || show_create_tag.get().then(|| view! {
                            <div class="create-form-inline">
                                <input
                                    type="text"
                                    placeholder="标签名称"
                                    prop:value=move || new_tag_name.get()
                                    on:input=move |ev| set_new_tag_name.set(event_target_value(&ev))
                                    class="input-sm"
                                />
                                <input
                                    type="color"
                                    prop:value=move || new_tag_color.get()
                                    on:input=move |ev| set_new_tag_color.set(event_target_value(&ev))
                                    class="input-color"
                                />
                                <button class="btn-sm" on:click=create_tag>"创建"</button>
                                <button class="btn-sm btn-cancel" on:click=move |_| set_show_create_tag.set(false)>"取消"</button>
                            </div>
                        })}

                        <div class="tag-list">
                            <For
                                each=move || tags.get()
                                key=|tag| tag.tag.id.clone()
                                children=|tag_with_count: TagWithCount| {
                                    let color = tag_with_count.tag.color.clone();
                                    view! {
                                        <div class="tag-item">
                                            <span class="tag-badge" style=format!("background-color: {}", color)>
                                                {&tag_with_count.tag.name}
                                            </span>
                                            <span class="tag-count">{tag_with_count.document_count}</span>
                                        </div>
                                    }
                                }
                            />
                        </div>
                    </div>
                </div>

                // 主内容区
                <div class="content-area">
                    {move || show_create_doc.get().then(|| {
                        let create_document = create_document.clone();
                        view! {
                            <div class="create-doc-form">
                                <h2>"创建新文档"</h2>
                                <input
                                    type="text"
                                    placeholder="文档标题"
                                    prop:value=move || new_doc_title.get()
                                    on:input=move |ev| set_new_doc_title.set(event_target_value(&ev))
                                    class="input-lg"
                                />
                                <div class="form-actions">
                                    <button class="btn btn-primary" on:click=create_document>"创建"</button>
                                    <button class="btn btn-secondary" on:click=move |_| set_show_create_doc.set(false)>"取消"</button>
                                </div>
                            </div>
                        }
                    })}

                    {
                        let nav_for_docs = navigate.clone();
                        move || if loading.get() {
                            view! { <div class="loading">"加载中..."</div> }.into_view()
                        } else if let Some(err) = error.get() {
                            view! { <div class="error">{err}</div> }.into_view()
                        } else {
                            let docs = documents.get();
                            if docs.is_empty() {
                                view! { <p class="empty">"还没有文档，点击右上角创建一个吧！"</p> }.into_view()
                            } else {
                                let nav = nav_for_docs.clone();
                                view! {
                                <div class="documents-grid">
                                    <For
                                        each=move || documents.get()
                                        key=|doc| doc.id.clone()
                                        children=move |doc: Document| {
                                            let doc_id = doc.id.clone();
                                            let nav_clone = nav.clone();
                                            view! {
                                                <div class="document-card" on:click=move |_| {
                                                    nav_clone(&format!("/editor/{}", doc_id), Default::default());
                                                }>
                                                    <div class="card-header">
                                                        <h3>{&doc.title}</h3>
                                                        {doc.is_public.then(|| view! {
                                                            <span class="badge badge-public">"公开"</span>
                                                        })}
                                                    </div>
                                                    <p class="card-preview">{
                                                        doc.content.chars().take(100).collect::<String>()
                                                    }</p>
                                                    <p class="card-meta">
                                                        <span>"创建于: "{doc.created_at.chars().take(10).collect::<String>()}</span>
                                                        <span>"更新于: "{doc.updated_at.chars().take(10).collect::<String>()}</span>
                                                    </p>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            }.into_view()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

// ===== 编辑器页面 =====

#[component]
fn EditorPage() -> impl IntoView {
    let params = use_params_map();
    let doc_id = move || params.with(|p| p.get("id").cloned().unwrap_or_default());

    let (title, set_title) = create_signal(String::new());
    let (content, set_content) = create_signal(String::new());
    let (loading, set_loading) = create_signal(true);
    let (saving, set_saving) = create_signal(false);
    let (error, set_error) = create_signal(None::<String>);

    let navigate = use_navigate();
    let navigate_clone = navigate.clone();

    // 加载文档
    create_effect(move |_| {
        let id = doc_id();
        if let Some(token) = get_token() {
            set_loading.set(true);
            spawn_local(async move {
                match fetch_document(&token, &id).await {
                    Ok(doc) => {
                        set_title.set(doc.title);
                        set_content.set(doc.content);
                        set_loading.set(false);
                    }
                    Err(e) => {
                        set_error.set(Some(e));
                        set_loading.set(false);
                    }
                }
            });
        } else {
            navigate_clone("/", Default::default());
        }
    });

    let save_doc = move |_| {
        let id = doc_id();
        if let Some(token) = get_token() {
            set_saving.set(true);
            let title_val = title.get();
            let content_val = content.get();

            spawn_local(async move {
                match update_document_api(&token, &id, title_val, content_val).await {
                    Ok(_) => {
                        set_error.set(Some("保存成功 ✓".to_string()));
                        set_timeout(move || set_error.set(None), std::time::Duration::from_secs(2));
                        set_saving.set(false);
                    }
                    Err(e) => {
                        set_error.set(Some(e));
                        set_saving.set(false);
                    }
                }
            });
        }
    };

    view! {
        <div class="editor-page">
            <div class="editor-toolbar">
                <button class="btn-back" on:click=move |_| navigate("/documents", Default::default())>
                    "← 返回"
                </button>
                <button class="btn" on:click=save_doc disabled=move || saving.get()>
                    {move || if saving.get() { "保存中..." } else { "💾 保存" }}
                </button>
            </div>

            {move || if loading.get() {
                view! { <div class="loading">"加载中..."</div> }.into_view()
            } else {
                view! {
                    <div class="editor-content">
                        <input
                            class="title-input"
                            type="text"
                            placeholder="文档标题"
                            prop:value=move || title.get()
                            on:input=move |ev| set_title.set(event_target_value(&ev))
                        />
                        <textarea
                            class="content-textarea"
                            placeholder="开始输入..."
                            prop:value=move || content.get()
                            on:input=move |ev| set_content.set(event_target_value(&ev))
                        />
                        {move || error.get().map(|e| view! {
                            <div class="message">{e}</div>
                        })}
                    </div>
                }.into_view()
            }}
        </div>
    }
}

// ===== 主应用 =====

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Router base="/">
            <Routes>
                <Route path="/" view=LoginPage/>
                <Route path="/register" view=RegisterPage/>
                <Route path="/documents" view=DocumentsPage/>
                <Route path="/editor/:id" view=EditorPage/>
            </Routes>
        </Router>
    }
}
