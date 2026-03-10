use std::collections::HashSet;

use iced::{
    Subscription, Task,
    widget::{
        Column, Row, button, text,
        text_editor::{self, Content},
    },
};

use crate::{
    db::{comments::Post, user::User},
    types::{String16, Timestamp, Topic},
    ui::{
        AppState,
        components::toast::{Toast, ToastType},
        message::Message,
        views::{View, ViewMessage},
    },
};

#[derive(Debug)]
pub struct PostView {
    topic: Topic,
    cur_page: usize,
    total_pages: usize,
    posts: Vec<Post>,
    users: HashSet<User>,

    content: Content,
}

impl Clone for PostView {
    fn clone(&self) -> Self {
        Self {
            topic: self.topic.clone(),
            cur_page: self.cur_page,
            total_pages: self.total_pages,
            posts: self.posts.clone(),
            users: self.users.clone(),
            content: Content::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum PostMessage {
    LoadPage(usize),
    LoadedPosts {
        posts: Vec<Post>,
        users: HashSet<User>,
        total_posts: usize,
    },
    AddPost,
    Posted(Post),
    EditComment(text_editor::Action),
}

impl From<PostMessage> for Message {
    fn from(msg: PostMessage) -> Message {
        Message::ViewMessage(ViewMessage::Post(msg))
    }
}

impl PostView {
    // TODO: Turn into option later
    const POST_PER_PAGE: usize = 50;

    pub fn new(topic: Topic) -> Self {
        Self {
            topic,
            cur_page: 1,
            total_pages: 1,
            posts: Vec::new(),
            users: HashSet::new(),
            content: Content::new(),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        Subscription::none()
    }

    pub fn on_enter(_state: &mut AppState) -> Task<Message> {
        Task::done(PostMessage::LoadPage(1).into())
    }

    pub fn view(&self, _state: &AppState) -> iced::Element<'_, Message> {
        let mut column = Column::new();

        for post in &self.posts {
            let profile = Column::new().push(text(match self.users.get(&post.source) {
                Some(user) => user.name(),
                None => "Unknown",
            }));

            column = column.push(
                Row::new()
                    .push(profile)
                    .push(text(post.content.inner().clone())),
            );
        }

        column = column
            .push(
                iced::widget::text_editor(&self.content)
                    .placeholder("Type something here...")
                    .on_action(|a| {
                        Message::ViewMessage(ViewMessage::Post(PostMessage::EditComment(a)))
                    }),
            )
            .push(button("Submit").on_press(PostMessage::AddPost.into()));

        column.into()
    }

    pub fn update(m: PostMessage, state: &mut AppState) -> Task<Message> {
        if let View::Post(v) = &mut state.view {
            match m {
                PostMessage::LoadedPosts {
                    posts,
                    users,
                    total_posts,
                } => {
                    v.posts = posts;
                    v.users = users;
                    v.total_pages = total_posts
                }
                PostMessage::EditComment(a) => {
                    v.content.perform(a);
                }
                PostMessage::AddPost => {
                    if let Some(repositories) = &state.repositories {
                        let repositories = repositories.clone();
                        let now = Timestamp::now();
                        let content = String16::new(v.content.text()).unwrap();
                        let post = Post::new_signed(
                            content,
                            now,
                            v.topic.clone(),
                            state.config.private_key(),
                        );
                        return Task::future(async move {
                            match repositories.add_post(post).await {
                                Ok(p) => PostMessage::Posted(p).into(),
                                Err(e) => Message::PostToast(Toast {
                                    title: "Failed to add post".to_string(),
                                    body: e.to_string(),
                                    ty: ToastType::Error,
                                }),
                            }
                        });
                    }
                }
                PostMessage::Posted(p) => {
                    if v.cur_page == v.total_pages {
                        v.posts.push(p);
                        v.content = Content::new();
                    }
                }
                PostMessage::LoadPage(page) => {
                    if page == 0 {
                        return Task::done(Message::PostToast(Toast {
                            title: "Cannot load page 0".to_string(),
                            body: "".to_string(),
                            ty: ToastType::Error,
                        }));
                    }

                    if let Some(repositories) = &state.repositories {
                        v.cur_page = page;
                        let repositories = repositories.clone();
                        let topic = v.topic.clone();
                        return Task::future(async move {
                            let res = match repositories
                                .get_posts_by_topic(
                                    topic,
                                    Self::POST_PER_PAGE,
                                    (page - 1) * Self::POST_PER_PAGE,
                                )
                                .await
                            {
                                Ok(res) => res,
                                Err(e) => {
                                    return Message::PostToast(Toast {
                                        title: "Failed to load posts".to_string(),
                                        body: e.to_string(),
                                        ty: ToastType::Error,
                                    });
                                }
                            };

                            PostMessage::LoadedPosts {
                                posts: res.values.0,
                                users: res.values.1,
                                total_posts: res.total,
                            }
                            .into()
                        });
                    }
                }
            }
        }
        Task::none()
    }
}
