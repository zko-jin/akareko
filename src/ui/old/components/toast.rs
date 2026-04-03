use futures::SinkExt;
use iced::{
    Element, stream,
    widget::{button, column, text},
};
use tokio::sync::mpsc;
use tracing::error;

use crate::{errors::DatabaseError, ui::message::Message};

#[derive(Debug, Clone)]
pub struct Toast {
    pub title: String,
    pub body: String,
    pub ty: ToastType,
}

impl Into<Message> for Toast {
    fn into(self) -> Message {
        Message::PostToast(self)
    }
}

pub enum ToastResult<T> {
    Ok(T),
    Err(Toast),
}

pub trait IntoToastResult<T> {
    fn to_toast(self, title: impl ToString) -> ToastResult<T>;
}

impl<T, E: std::fmt::Display> IntoToastResult<T> for Result<T, E> {
    fn to_toast(self, title: impl ToString) -> ToastResult<T> {
        match self {
            Ok(v) => ToastResult::Ok(v),
            Err(e) => ToastResult::Err(Toast::error(title.to_string(), e.to_string())),
        }
    }
}

impl<T> core::ops::FromResidual for ToastResult<T> {
    fn from_residual(residual: <Self as core::ops::Try>::Residual) -> Self {
        match residual {
            Message::PostToast(toast) => ToastResult::Err(toast),
            _ => unreachable!(),
        }
    }
}

impl core::ops::FromResidual<Message> for Message {
    fn from_residual(residual: Self) -> Self {
        residual
    }
}

impl<T> std::ops::Try for ToastResult<T> {
    type Output = T;
    type Residual = Message;

    fn from_output(output: Self::Output) -> Self {
        ToastResult::Ok(output)
    }

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self {
            ToastResult::Ok(v) => std::ops::ControlFlow::Continue(v),
            ToastResult::Err(e) => std::ops::ControlFlow::Break(e.into()),
        }
    }
}

impl Toast {
    pub fn error(title: impl ToString, body: impl ToString) -> Self {
        Self {
            title: title.to_string(),
            body: body.to_string(),
            ty: ToastType::Error,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ToastType {
    Info,
    Error,
}

impl Toast {
    pub fn view(&self, index: usize) -> Element<'_, Message> {
        column![
            button(text("X")).on_press(Message::CloseToast(index)),
            text(&self.title),
            text(&self.body)
        ]
        .into()
    }
}

pub fn toast_worker() -> impl iced::futures::Stream<Item = Message> {
    stream::channel(
        100,
        |mut output: futures::channel::mpsc::Sender<Message>| async move {
            let (tx, mut rx) = mpsc::channel::<Toast>(100);
            match output.send(Message::ToastSenderReady(tx)).await {
                Ok(()) => {}
                Err(e) => {
                    // This should honestly never happen, it's here just in case
                    error!("Error initializing toast subscriptions: {}", e);
                }
            };

            loop {
                let toast = match rx.recv().await {
                    Some(toast) => toast,
                    None => break,
                };

                match output.send(Message::PostToast(toast)).await {
                    Ok(()) => {}
                    Err(e) => {
                        if e.is_disconnected() {
                            error!("Disconnected from toast output");
                        } else if e.is_full() {
                            error!("Toast output is full");
                        }
                    }
                };
            }
        },
    )
}
