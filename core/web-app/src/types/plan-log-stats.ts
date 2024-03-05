export interface PlanLogStats {
  numberOfEvents: number;
  numberOfDailyEventsByPlan: {
    [planId: string]: DailyEvent[];
  };
}

export interface DailyEvent {
  date: string;
  count: number;
}
