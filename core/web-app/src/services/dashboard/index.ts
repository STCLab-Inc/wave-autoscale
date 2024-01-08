import { DataLayer } from '@/infra/data-layer';
import dayjs from 'dayjs';
import MetricService from '../metric';
import ScalingComponentService from '../scaling-component';
import ScalingPlanService from '../scaling-plan';
import _ from 'lodash';

export interface DashboardStats {
  autoscalingHistoryCount: number;
  autoscalingHistoryMostTriggered: [[string, number]];
  metricsCount: number;
  scalingComponentsCount: number;
  plansCount: number;
}

class DashboardServiceClass {
  async getDashboardStats(): Promise<DashboardStats> {
    const to = dayjs();
    const from = to.subtract(7, 'day');

    const getAutoscalingHistory = async () => {
      const autoscalingHistory = await DataLayer.get(
        `/api/autoscaling-history?from=${
          from.format('YYYY-MM-DDTHH:mm:ss') + '.000Z'
        }&to=${to.format('YYYY-MM-DDTHH:mm:ss') + '.000Z'}`
      );
      return autoscalingHistory;
    };

    const getAutoscalingHistoryCount = async () => {
      const autoscalingHistory = await getAutoscalingHistory();
      const autoscalingHistoryCount = autoscalingHistory.data?.length;
      return autoscalingHistoryCount;
    };
    const getAutoscalingHistoryMostTriggered = async () => {
      const autoscalingHistory = await getAutoscalingHistory();
      const { data } = autoscalingHistory;
      const ordered = _.chain(data)
        .groupBy('plan_id')
        .mapValues((value) => value.length)
        .toPairs()
        .orderBy(1, 'desc')
        .slice(0,10)
        .value();
      return ordered;
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
      autoscalingHistoryMostTriggered,
      metricsCount,
      scalingComponentsCount,
      plansCount,
    ] = await Promise.all([
      getAutoscalingHistoryCount(),
      getAutoscalingHistoryMostTriggered(),
      getMetricsCount(),
      getScalingComponentsCount(),
      getPlansCount(),
    ]);

    return {
      autoscalingHistoryCount,
      autoscalingHistoryMostTriggered,
      metricsCount,
      scalingComponentsCount,
      plansCount,
    };
  }
}

const DashboardService = new DashboardServiceClass();

export default DashboardService;
