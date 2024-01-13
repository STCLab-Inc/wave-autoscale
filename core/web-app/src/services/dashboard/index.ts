import { DataLayer } from '@/infra/data-layer';
import dayjs from 'dayjs';
import MetricService from '../metric';
import ScalingComponentService from '../scaling-component';
import ScalingPlanService from '../scaling-plan';
import _ from 'lodash';
import { decodeTime } from 'ulid';

export interface DashboardStats {
  autoscalingHistoryCount: number;
  autoscalingHistoryMostTriggered: [string, number][];
  dailyAutoscalingHistory: { x: string; y: number }[];
  dailyAutoscalingHistoryKeys: string[];
  metricsCount: number;
  scalingComponentsCount: number;
  plansCount: number;
}

class DashboardServiceClass {
  async getDashboardStats(): Promise<DashboardStats> {
    const to = dayjs();
    const from = to.subtract(14, 'day');

    const getAutoscalingHistoryStats = async () => {
      const autoscalingHistory = await DataLayer.get(
        `/api/autoscaling-history?from=${
          from.format('YYYY-MM-DDTHH:mm:ss') + '.000Z'
        }&to=${to.format('YYYY-MM-DDTHH:mm:ss') + '.000Z'}`
      );
      const { data } = autoscalingHistory;
      const count = data?.length;
      const ordered = _.chain(data)
        .groupBy('plan_id')
        .mapValues((value) => value.length)
        .toPairs()
        .orderBy(1, 'desc')
        .slice(0, 10)
        .value();

      const dailyDates = [];
      for (let i = from; !i.isAfter(to); i = i.add(1, 'day')) {
        dailyDates.push(i.format('YYYY-MM-DD'));
      }

      const parsedForDaily = _.chain(data)
        .groupBy((value) => dayjs(decodeTime(value.id)).format('YYYY-MM-DD'))
        .mapValues((value) => value.length)
        .value();

      const daily = _.map(dailyDates, (date) => {
        return {
          date,
          value: parsedForDaily[date] || 0,
        };
      });

      return {
        count,
        ordered,
        daily,
        dailyKeys: dailyDates,
      };
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
      autoscalingHistoryStats,
      metricsCount,
      scalingComponentsCount,
      plansCount,
    ] = await Promise.all([
      getAutoscalingHistoryStats(),
      getMetricsCount(),
      getScalingComponentsCount(),
      getPlansCount(),
    ]);

    return {
      autoscalingHistoryCount: autoscalingHistoryStats.count,
      autoscalingHistoryMostTriggered: autoscalingHistoryStats.ordered,
      dailyAutoscalingHistory: autoscalingHistoryStats.daily,
      dailyAutoscalingHistoryKeys: autoscalingHistoryStats.dailyKeys,
      metricsCount,
      scalingComponentsCount,
      plansCount,
    };
  }
}

const DashboardService = new DashboardServiceClass();

export default DashboardService;
