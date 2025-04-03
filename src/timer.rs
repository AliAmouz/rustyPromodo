use std::time::{Duration, Instant};

#[derive(PartialEq, Copy, Clone)]
pub enum TimerState {
    Running,
    Paused,
    Stopped,
}

#[derive(PartialEq, Copy, Clone)]
pub enum TimerType {
    Work,
    Break,
}

pub struct PomodoroTimer {
    work_duration: Duration,
    break_duration: Duration,
    start_time: Option<Instant>,
    pause_time: Option<Instant>,
    elapsed_before_pause: Duration,
    timer_type: TimerType,
    state: TimerState,
}

impl PomodoroTimer {
    pub fn new(work_duration: Duration, break_duration: Duration) -> Self {
        Self {
            work_duration,
            break_duration,
            start_time: None,
            pause_time: None,
            elapsed_before_pause: Duration::from_secs(0),
            timer_type: TimerType::Work,
            state: TimerState::Stopped,
        }
    }
    
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.elapsed_before_pause = Duration::from_secs(0);
        self.state = TimerState::Running;
    }
    
    pub fn pause(&mut self) {
        if self.state == TimerState::Running {
            self.pause_time = Some(Instant::now());
            self.state = TimerState::Paused;
        }
    }
    
    pub fn resume(&mut self) {
        if self.state == TimerState::Paused {
            if let (Some(start), Some(pause)) = (self.start_time, self.pause_time) {
                self.elapsed_before_pause += pause.duration_since(start);
                self.start_time = Some(Instant::now());
                self.pause_time = None;
            }
            self.state = TimerState::Running;
        }
    }
    
    pub fn reset(&mut self) {
        self.start_time = Some(Instant::now());
        self.pause_time = None;
        self.elapsed_before_pause = Duration::from_secs(0);
        if self.state != TimerState::Stopped {
            self.state = TimerState::Running;
        }
    }
    
    pub fn switch_to_work(&mut self) {
        self.timer_type = TimerType::Work;
        self.reset();
    }
    
    pub fn switch_to_break(&mut self) {
        self.timer_type = TimerType::Break;
        self.reset();
    }
    
    pub fn update(&mut self) {
        // Update internal timer state if needed
    }
    
    pub fn elapsed(&self) -> Duration {
        match (self.state, self.start_time, self.pause_time) {
            (TimerState::Running, Some(start), _) => {
                self.elapsed_before_pause + start.elapsed()
            }
            (TimerState::Paused, Some(start), Some(pause)) => {
                self.elapsed_before_pause + pause.duration_since(start)
            }
            _ => self.elapsed_before_pause,
        }
    }
    
    pub fn total_time(&self) -> Duration {
        match self.timer_type {
            TimerType::Work => self.work_duration,
            TimerType::Break => self.break_duration,
        }
    }
    
    pub fn is_complete(&self) -> bool {
        self.elapsed() >= self.total_time()
    }
    
    pub fn timer_type(&self) -> TimerType {
        self.timer_type
    }
    
    pub fn state(&self) -> TimerState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    
    #[test]
    fn test_timer_creation() {
        let timer = PomodoroTimer::new(
            Duration::from_secs(25 * 60),
            Duration::from_secs(5 * 60),
        );
        
        assert_eq!(timer.state, TimerState::Stopped);
        assert_eq!(timer.timer_type, TimerType::Work);
        assert_eq!(timer.total_time(), Duration::from_secs(25 * 60));
    }
    
    #[test]
    fn test_timer_elapsed() {
        let mut timer = PomodoroTimer::new(
            Duration::from_secs(25 * 60),
            Duration::from_secs(5 * 60),
        );
        
        timer.start();
        sleep(Duration::from_millis(100));
        
        let elapsed = timer.elapsed();
        assert!(elapsed.as_millis() >= 50, "Timer elapsed should be at least 50ms");
    }
    
    #[test]
    fn test_switch_timer_type() {
        let mut timer = PomodoroTimer::new(
            Duration::from_secs(25 * 60),
            Duration::from_secs(5 * 60),
        );
        
        assert_eq!(timer.timer_type(), TimerType::Work);
        assert_eq!(timer.total_time(), Duration::from_secs(25 * 60));
        
        timer.switch_to_break();
        assert_eq!(timer.timer_type(), TimerType::Break);
        assert_eq!(timer.total_time(), Duration::from_secs(5 * 60));
    }
}