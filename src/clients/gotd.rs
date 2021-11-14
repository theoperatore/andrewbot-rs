use chrono::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GameImage {
  pub original_url: Option<String>,
  pub super_url: Option<String>,
  pub screen_url: Option<String>,
  pub screen_large_url: Option<String>,
  pub medium_url: Option<String>,
  pub small_url: Option<String>,
  pub thumb_url: Option<String>,
  pub icon_url: Option<String>,
  pub tiny_url: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Characteristic {
  api_detail_url: String,
  id: i32,
  pub name: String,
  site_detail_url: String,
  abbreviation: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct Game {
  id: i32,
  guid: String,
  pub image: Option<GameImage>,
  pub name: String,
  pub deck: Option<String>,
  description: Option<String>,
  pub original_release_date: Option<String>,
  pub site_detail_url: Option<String>,
  pub expected_release_day: Option<i32>,
  pub expected_release_month: Option<i32>,
  pub expected_release_year: Option<i32>,
  pub expected_release_quarter: Option<i32>,
  pub platforms: Option<Vec<Characteristic>>,
  concepts: Option<Vec<Characteristic>>,
  developers: Option<Vec<Characteristic>>,
  characters: Option<Vec<Characteristic>>,
  themes: Option<Vec<Characteristic>>,
}

#[derive(Deserialize, Debug)]
struct AlorgResponse {
  status: String,
  result: Game, // error message
}

pub async fn get_random_game() -> Result<Game, reqwest::Error> {
  let url = "https://datas.alorg.net/api/v1/games/random";

  let mut res = reqwest::get(url).await;
  if let Err(why) = res {
    tracing::error!("Failed game query, trying again: {}", why);
    res = reqwest::get(url).await;
  }

  let parsed = res?.json::<AlorgResponse>().await?;
  Ok(parsed.result)
}

pub fn parse_image(game: &Game) -> String {
  match &game.image {
    Some(image) => match image
      .super_url
      .as_ref()
      .or(image.screen_url.as_ref())
      .or(image.medium_url.as_ref())
      .or(image.small_url.as_ref())
      .or(image.thumb_url.as_ref())
      .or(image.icon_url.as_ref())
      .or(image.tiny_url.as_ref())
    {
      Some(i) => i.to_string(),
      None => String::from(""),
    },
    None => String::from(""),
  }
}

pub fn parse_date(game: &Game) -> String {
  let first = game
    .original_release_date
    .as_ref()
    .map(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
    .flatten()
    .map(|dt| dt.format("%b %e, %Y").to_string());

  let second = match (
    game.expected_release_year.as_ref(),
    game.expected_release_month.as_ref(),
    game.expected_release_day.as_ref(),
  ) {
    (Some(y), Some(m), Some(d)) => {
      let datestr = format!("{}-{}-{}", y, m, d);
      NaiveDate::parse_from_str(&datestr, "%Y-%m-%d").ok()
    }
    _ => None,
  }
  .map(|dt| dt.format("%b %e, %Y").to_string());

  let third = match (
    game.expected_release_year.as_ref(),
    game.expected_release_month.as_ref(),
  ) {
    (Some(y), Some(m)) => {
      let datestr = format!("{}-{}-01", y, m);
      NaiveDate::parse_from_str(&datestr, "%Y-%m-%d").ok()
    }
    _ => None,
  }
  .map(|dt| dt.format("%b %Y").to_string());

  let fourth = match (
    game.expected_release_year.as_ref(),
    game.expected_release_quarter.as_ref(),
  ) {
    (Some(y), Some(q)) => Some(format!("Q{} {}", q, y)),
    _ => None,
  };

  first
    .or(second)
    .or(third)
    .or(fourth)
    .or(game.expected_release_year.map(|y| format!("{}", y)))
    .unwrap_or(String::from("No date listed"))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_date_parses_original_date() {
    let mut original_date_game = Game::default();
    original_date_game.original_release_date = Some(String::from("2021-03-10"));
    assert_eq!(parse_date(&original_date_game), "Mar 10, 2021");
  }

  #[test]
  fn parse_date_parses_expected_ymd_date() {
    let mut game = Game::default();
    game.expected_release_year = Some(2021);
    game.expected_release_month = Some(3);
    game.expected_release_day = Some(10);
    assert_eq!(parse_date(&game), "Mar 10, 2021");
  }

  #[test]
  fn parse_date_parses_expected_ym_date() {
    let mut game = Game::default();
    game.expected_release_year = Some(2021);
    game.expected_release_month = Some(3);
    assert_eq!(parse_date(&game), "Mar 2021");
  }

  #[test]
  fn parse_date_parses_expected_y_quarter_date() {
    let mut game = Game::default();
    game.expected_release_year = Some(2021);
    game.expected_release_quarter = Some(3);
    assert_eq!(parse_date(&game), "Q3 2021");
  }

  #[test]
  fn parse_date_parses_expected_y_date() {
    let mut game = Game::default();
    game.expected_release_year = Some(2021);
    assert_eq!(parse_date(&game), "2021");
  }

  #[test]
  fn parse_date_returns_no_date_listed_when_all_none() {
    let game = Game::default();
    assert_eq!(parse_date(&game), "No date listed");
  }
}
