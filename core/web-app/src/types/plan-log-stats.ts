export interface PlanLogStats {
  totalCount: number;
  countPerDayByPlanId: {
    [planId: string]: PlanLogCountPerDay[];
  };
}

export interface PlanLogCountPerDay {
  date: string;
  count: number;
}
