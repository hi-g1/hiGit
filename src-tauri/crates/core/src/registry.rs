use crate::{Action, ActionMeta, CoreError, Result};

/// 등록된 액션 목록. 화면의 버튼·메뉴는 [`Registry::metas`]로 만든다(D-005, D-010).
#[derive(Default)]
pub struct Registry {
    actions: Vec<Box<dyn Action>>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, action: Box<dyn Action>) -> Result<()> {
        let id = action.meta().id;
        if self.get(id).is_some() {
            return Err(CoreError::DuplicateAction(id));
        }
        self.actions.push(action);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&dyn Action> {
        self.actions
            .iter()
            .find(|a| a.meta().id == id)
            .map(|a| a.as_ref())
    }

    pub fn metas(&self) -> impl Iterator<Item = &ActionMeta> {
        self.actions.iter().map(|a| a.meta())
    }
}
