import { DataLayer } from '@/infra/data-layer';
import { PlanLogStats } from '@/types/plan-log-stats';
import { PlanLogDefinition } from '@/types/bindings/plan-log-definition';
import { parseUlidToDayjs } from '@/utils/date';
import { useQuery } from '@tanstack/react-query';
import { Dayjs } from 'dayjs';
import { forEach, groupBy } from 'lodash';
import queryString from 'query-string';
import { PlanLogDefinitionEx } from '@/types/plan-log-definition-ex';

class PlanLogServiceClass {
  async getPlanLogsByFromTo(
    planId: string | undefined,
    from: Dayjs,
    to: Dayjs
  ): Promise<PlanLogDefinitionEx[]> {
    const query: any = {
      // Unix timestamp in milliseconds
      from: from.valueOf(),
      to: to.valueOf(),
    };
    if (planId) {
      query['plan_id'] = planId;
    }
    const { data } = await DataLayer.get(
      `/api/plan-logs?${queryString.stringify(query)}`
    );
    const result = data.map((item: PlanLogDefinition) => {
      return {
        ...item,
        created_at: parseUlidToDayjs(item.id)?.valueOf(),
      };
    });

    return result;
  }
  async getPlanLogStats(from: Dayjs, to: Dayjs): Promise<PlanLogStats> {
    const data = await this.getPlanLogsByFromTo(undefined, from, to);
    const stats: PlanLogStats = {
      totalCount: data.length,
      countPerDayByPlanId: {},
    };

    const groupedData = groupBy(data, 'plan_id');
    forEach(groupedData, (historyItems, planId) => {
      // Calculate the number of events for each plan daily
      stats.countPerDayByPlanId[planId] = [];
      // Initialize the array
      let currentDate = from;
      while (currentDate.isBefore(to) || currentDate.isSame(to)) {
        stats.countPerDayByPlanId[planId].push({
          date: currentDate.format('YYYY-MM-DD'),
          count: 0,
        });
        currentDate = currentDate.add(1, 'day');
      }
      // Count the number of events for each day
      forEach(historyItems, (historyItem) => {
        const dateString = parseUlidToDayjs(historyItem.id)?.format(
          'YYYY-MM-DD'
        );
        const index = stats.countPerDayByPlanId[planId].findIndex(
          (item: any) => item.date === dateString
        );
        if (index !== -1) {
          stats.countPerDayByPlanId[planId][index].count++;
        }
      });
    });
    return stats;
  }
}

const PlanLogService = new PlanLogServiceClass();

export default PlanLogService;

// Hooks
function usePlanLogs(planId: string | undefined, from: Dayjs, to: Dayjs) {
  return useQuery({
    queryKey: ['plan-log', from, to],
    queryFn: () => PlanLogService.getPlanLogsByFromTo(planId, from, to),
  });
}

function usePlanLogStats(from: Dayjs, to: Dayjs) {
  return useQuery({
    queryKey: ['plan-log-stats', from, to],
    queryFn: () => PlanLogService.getPlanLogStats(from, to),
  });
}

export { usePlanLogs, usePlanLogStats };
