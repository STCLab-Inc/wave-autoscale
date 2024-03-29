import { DataLayer } from '@/infra/data-layer';
import StatsService from '../stats';
import {
  ScalingPlanDefinitionEx,
  ScalingPlanWithStats,
} from '@/types/scaling-plan-definition-ex';
import { useQuery } from '@tanstack/react-query';
import PlanLogService from '../plan-log';
import { Dayjs } from 'dayjs';

class ScalingPlanServiceClass {
  async getScalingPlans(): Promise<ScalingPlanDefinitionEx[]> {
    const response = await DataLayer.get('/api/plans');
    return response.data;
  }

  async getScalingPlan(id: string): Promise<ScalingPlanDefinitionEx> {
    const plans = await this.getScalingPlans();
    return plans.find((plan) => plan.id === id) as ScalingPlanDefinitionEx;
  }

  async getScalingPlanByDbId(db_id: string) {
    const response = await DataLayer.get(`/api/plans/${db_id}`);
    return response.data;
  }

  async syncScalingPlanYaml(yaml: string) {
    const response = await DataLayer.post('/api/plans/yaml', {
      yaml,
    });
    await StatsService.invalidateStats();
    return response.data;
  }

  async deleteScalingPlan(id: string) {
    const response = await DataLayer.delete(`/api/plans/${id}`);
    await StatsService.invalidateStats();
    return response.data;
  }
}

const ScalingPlanService = new ScalingPlanServiceClass();

export default ScalingPlanService;

// Hooks

function useScalingPlans() {
  return useQuery({
    queryKey: ['scaling-plans'],
    queryFn: ScalingPlanService.getScalingPlans,
  });
}

function useScalingPlan(id: string) {
  return useQuery({
    queryKey: ['scaling-plan', id],
    queryFn: async () => {
      return ScalingPlanService.getScalingPlan(id);
    },
  });
}

function useScalingPlansWithStats(from: Dayjs, to: Dayjs) {
  return useQuery<ScalingPlanWithStats[]>({
    queryKey: ['scaling-plans-with-stats', from.valueOf(), to.valueOf()],
    queryFn: async () => {
      const [plans, stats] = await Promise.all([
        ScalingPlanService.getScalingPlans(),
        // getPlanLogStats returns all plan logs, so we need to filter them by planId later
        PlanLogService.getPlanLogStats(from, to),
      ]);

      const plansWithStats = plans.map((plan) => {
        const countPerDay = stats.countPerDayByPlanId[plan.id] || [];
        return { ...plan, countPerDay } as ScalingPlanWithStats;
      });
      return plansWithStats;
    },
  });
}

export { useScalingPlans, useScalingPlansWithStats, useScalingPlan };
