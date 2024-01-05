import { DataLayer } from '@/infra/data-layer';
import dayjs from 'dayjs';
import MetricService from '../metric';
import ScalingComponentService from '../scaling-component';
import ScalingPlanService from '../scaling-plan';
import InflowService from '../inflow';

export interface DashboardStats {
  autoscalingHistoryCount: number;
  metricsCount: number;
  scalingComponentsCount: number;
  plansCount: number;
}

class DashboardServiceClass {
  async getDashboardStats(): Promise<DashboardStats> {
    const to = dayjs();
    const from = to.subtract(7, 'day');

    const getAutoscalingHistoryCount = async () => {
      const autoscalingHistory = await DataLayer.get(
        `/api/autoscaling-history?from=${
          from.format('YYYY-MM-DDTHH:mm:ss') + '.000Z'
        }&to=${to.format('YYYY-MM-DDTHH:mm:ss') + '.000Z'}`
      );
      const autoscalingHistoryCount = autoscalingHistory.data.length;
      return autoscalingHistoryCount;
    };

    const getMetricsCount = async () => {
      const metrics = await MetricService.getMetrics();
      const metricsCount = metrics.length;
      return metricsCount;
    };

    const getScalingComponentsCount = async () => {
      const scalingComponents =
        await ScalingComponentService.getScalingComponents();
      const scalingComponentsCount = scalingComponents.length;
      return scalingComponentsCount;
    };

    const getPlansCount = async () => {
      const plans = await ScalingPlanService.getScalingPlans();
      const plansCount = plans.length;
      return plansCount;
    };

    const [
      autoscalingHistoryCount,
      metricsCount,
      scalingComponentsCount,
      plansCount,
    ] = await Promise.all([
      getAutoscalingHistoryCount(),
      getMetricsCount(),
      getScalingComponentsCount(),
      getPlansCount(),
    ]);

    return {
      autoscalingHistoryCount,
      metricsCount,
      scalingComponentsCount,
      plansCount,
    };
  }
}

const DashboardService = new DashboardServiceClass();

export default DashboardService;