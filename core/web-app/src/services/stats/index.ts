import { DataLayer } from '@/infra/data-layer';
import dayjs from 'dayjs';
import MetricService from '../metric';
import ScalingComponentService from '../scaling-component';
import ScalingPlanService from '../scaling-plan';
import _ from 'lodash';
import { decodeTime } from 'ulid';
import { invalidateQuery } from '@/infra/query-client';
import { useQuery } from '@tanstack/react-query';
import queryString from 'query-string';

export interface DashboardStats {
  autoscalingHistoryCount: number;
  autoscalingHistoryMostTriggered: [string, number][];
  dailyAutoscalingHistory: { [key: string]: any }[];
  dailyAutoscalingHistoryKeys: string[];
  dailyAutoscalingHistoryPlanIds: string[];
}

export interface MenuStats {
  metricsCount: number;
  scalingComponentsCount: number;
  scalingPlansCount: number;
}

class StatsServiceClass {
  async getDashboardStats(): Promise<DashboardStats> {
    const to = dayjs();
    const from = to.subtract(13, 'day');

    const getPlanLogStats = async () => {
      const query = {
        from: from.valueOf(),
        to: to.valueOf(),
      };
      const planLogs = await DataLayer.get(
        `/api/plan-logs?${queryString.stringify(query)}`
      );
      const { data } = planLogs;
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
        dailyDates.push(i.format('MM-DD'));
      }

      const dailyPlanIds = _.chain(data).map('plan_id').uniq().value();
      const parsedForDaily = _.chain(data)
        .groupBy((value) => dayjs(decodeTime(value.id)).format('MM-DD'))
        .mapValues((valueItems) => {
          return _.chain(valueItems)
            .groupBy('plan_id')
            .mapValues((value) => value.length)
            .value();
        })
        .map((value, key) => {
          return {
            date: key,
            ...value,
          };
        })
        .value();

      const daily = _.map(dailyDates, (date) => {
        const found = _.find(parsedForDaily, { date });
        if (found) {
          return found;
        }
        return {
          date,
        };
      });

      return {
        count,
        ordered,
        daily,
        dailyKeys: dailyDates,
        dailyPlanIds,
      };
    };

    const [planLogStats] = await Promise.all([getPlanLogStats()]);

    return {
      autoscalingHistoryCount: planLogStats.count,
      autoscalingHistoryMostTriggered: planLogStats.ordered,
      dailyAutoscalingHistory: planLogStats.daily,
      dailyAutoscalingHistoryKeys: planLogStats.dailyKeys,
      dailyAutoscalingHistoryPlanIds: planLogStats.dailyPlanIds,
    };
  }
  async getMenuStats(): Promise<MenuStats> {
    const [metrics, scalingComponents, scalingPlans] = await Promise.all([
      MetricService.getMetrics(),
      ScalingComponentService.getScalingComponents(),
      ScalingPlanService.getScalingPlans(),
    ]);
    return {
      metricsCount: metrics.length,
      scalingComponentsCount: scalingComponents.length,
      scalingPlansCount: scalingPlans.length,
    };
  }
  async invalidateStats() {
    return invalidateQuery(['stats']);
  }
}

const StatsService = new StatsServiceClass();

function useMenuStats() {
  return useQuery({
    queryKey: ['stats'],
    queryFn: StatsService.getMenuStats,
  });
}

export default StatsService;

export { useMenuStats };
