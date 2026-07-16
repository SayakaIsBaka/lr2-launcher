use slint::{Model, ModelRc, SharedString, VecModel, language::StandardListViewItem};
use anyhow::Result;
use crate::ApplicationGlobal;

pub fn find_player_in_array(players: &Vec<SharedString>, username: &String) -> Option<usize> {
    players.iter().position(|x| x.as_str() == username)
}

pub fn slint_arr_to_jukebox_paths(app_globals: &ApplicationGlobal) -> Vec<String> {
    let mut new_paths: Vec<String> = vec![];
    for path in app_globals.get_jukebox_paths().iter() {
        new_paths.push(path.text.into());
    }
    new_paths
}

pub fn jukebox_paths_to_slint_arr(paths: &Option<Vec<String>>) -> Result<ModelRc<StandardListViewItem>> {
    let mut standard_list_view_vec: Vec<StandardListViewItem> = vec![];
    if paths.is_some() {
        for path in paths.as_ref().unwrap() {
            standard_list_view_vec.push(StandardListViewItem::from(path.as_str()));
        }
    }
    Ok(VecModel::from_slice(standard_list_view_vec.as_slice()))
}