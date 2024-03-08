import { DataLayer } from '@/infra/data-layer';
import {
  MetricsDataItem,
  MetricsDataItems,
  MetricsDataStats,
  MetricsDataStatsMap,
} from '@/types/metrics-data-stats';
import { parseDateToDayjs } from '@/utils/date';
import { useQuery } from '@tanstack/react-query';
import dayjs, { Dayjs } from 'dayjs';
import _ from 'lodash';
import queryString from 'query-string';

class MetricsDataServiceClass {
  async getStats(): Promise<MetricsDataStats[]> {
    const { data }: { data: MetricsDataStatsMap } = await DataLayer.get(
      `/api/metrics-data/stats`
    );

    let metricsDataStats: MetricsDataStats[] = _.map(
      data,
      (value, metricId) => {
        const timestamps = value[0];
        // lastValues is a JSON string including metrics data
        const lastValues = value[1];
        // The number of values is the length of the lastValues array
        let numberOfValues = 0;

        try {
          numberOfValues = JSON.parse(lastValues).length;
        } catch (e) {
          console.error(e);
        }

        // Frequency in minutes last 5 minutes
        const MINUTES_AGO = dayjs().subtract(5, 'minute');
        const countPerMinuteMap: { [key: string]: number } = {};

        // Initialize countPerMinuteMap with 0 for each minute
        for (let i = 0; i < dayjs().diff(MINUTES_AGO, 'minute'); i++) {
          countPerMinuteMap[i.toString()] = 0;
        }

        timestamps.forEach((timestamp: number) => {
          const dayjsTimestamp = parseDateToDayjs(timestamp);
          if (!dayjsTimestamp) {
            return;
          }
          const diffMinutes = dayjsTimestamp
            .diff(MINUTES_AGO, 'minute')
            .toString();
          countPerMinuteMap[diffMinutes] = countPerMinuteMap[diffMinutes]
            ? countPerMinuteMap[diffMinutes] + 1
            : 1;
        });
        const countPerMinute: MetricsDataStats['countPerMinute'] = _.map(
          countPerMinuteMap,
          (value, key) => {
            return {
              minute: key,
              count: value,
            };
          }
        );

        return {
          metricId,
          timestamps,
          countPerMinute,
          numberOfValues,
          lastValues,
        };
      }
    );

    // Sort by metricId for consistency
    metricsDataStats = _.sortBy(metricsDataStats, (item) => item.metricId);
    return metricsDataStats;
  }

  async getMetricsData(
    metricId: string,
    from?: Dayjs,
    to?: Dayjs
  ): Promise<MetricsDataItems> {
    const query: any = {};
    if (from) {
      query['from'] = from.valueOf();
    }
    if (to) {
      query['to'] = to.valueOf();
    }
    const { data }: { data: { [ulid: string]: MetricsDataItem[] } } =
      await DataLayer.get(
        `/api/metrics-data/metrics/${metricId}?${queryString.stringify(query)}`
      );
    const itemsMap: { [key: string]: MetricsDataItem[] } = {};
    _.forEach(data, (values: any[], timestamp: string) => {
      values.forEach((value: any, index: number) => {
        const keyObject: any = {};
        if (value.name) {
          keyObject.name = value.name;
        }
        if (value.tags) {
          keyObject.tags = value.tags;
        }
        const key = JSON.stringify(keyObject);
        if (!itemsMap[key]) {
          itemsMap[key] = [];
        }
        itemsMap[key].push(value);
      });
    });
    const metricsDataItems: MetricsDataItems = _.map(
      itemsMap,
      (values, key) => {
        return {
          ...JSON.parse(key),
          values,
        };
      }
    );
    return metricsDataItems;
  }
}

const MetricsDataService = new MetricsDataServiceClass();

export default MetricsDataService;

// Hooks

function useMetricsDataStats() {
  return useQuery({
    queryKey: ['metrics-data-stats'],
    queryFn: MetricsDataService.getStats,
  });
}

function useMetricsData(metricId: string, from?: Dayjs, to?: Dayjs) {
  return useQuery({
    queryKey: ['metrics-data', metricId, from?.valueOf(), to?.valueOf()],
    queryFn: () => MetricsDataService.getMetricsData(metricId, from, to),
  });
}

export { useMetricsDataStats, useMetricsData };
