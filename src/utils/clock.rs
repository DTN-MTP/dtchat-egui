use dtchat_backend::time::DTChatTime;

pub struct Clock {
    minutes: u32,
    hours: u32,
    str: String,
    anim: Option<DTChatTime>,
}
impl Clock {
    fn clock(hours: u32, mins: u32) -> char {
        let mut idx: u32 = hours % 12;
        if mins >= 45 {
            idx += 1;
        }

        if idx == 0 {
            idx = 12;
        }

        if mins >= 15 && mins < 45 {
            idx += 12;
        };
        char::from_u32(0x1F54F + idx).unwrap()
    }

    pub fn new(dt: &DTChatTime, anim: bool) -> Self {
        let (mins, hours) = dt.mins_hours(&chrono::Local);
        Self {
            minutes: mins,
            hours: hours,
            str: format!("{}", Clock::clock(hours, mins)),
            anim: if anim { Some(dt.clone()) } else { None },
        }
    }

    pub fn switch_anim(&mut self, curr_time: &DTChatTime) {
        match self.anim {
            Some(_) => {
                self.minutes = 100;
                self.anim = None
            }
            None => self.anim = Some(curr_time.clone()),
        }
    }
    pub fn update(&mut self, current_time: &DTChatTime) {
        if let Some(last_anim) = self.anim {
            if (current_time.timestamp_millis() - last_anim.timestamp_millis()) > 100 {
                self.hours += 1;
                self.hours %= 24;
                self.str = format!("{}", Clock::clock(self.hours, self.minutes));
                self.anim = Some(current_time.clone());
            }
        } else {
            let (mins, hours) = current_time.mins_hours(&chrono::Local);
            if self.minutes != mins {
                self.minutes = mins;
                self.hours = hours;
                self.str = format!("{}", Clock::clock(hours, mins));
            }
        };
    }
    pub fn to_string(&self) -> String {
        self.str.clone()
    }
}
