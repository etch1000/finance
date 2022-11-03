use crate::YahooFinanceQuote;
use chrono::{DateTime, Local, NaiveDateTime, NaiveTime, Utc};
use comfy_table::{Cell, CellAlignment, Color, Table};
use itertools::Itertools;
use serde::Deserialize;
use std::{collection::HashMap, fmt::{Display, Formatter}};
