import { DataLayer } from '@/infra/data-layer';
import { MetricsDataStats } from '@/types/metrics-data-stats';
import { parseDateToDayjs } from '@/utils/date';
import { useQuery } from '@tanstack/react-query';
import dayjs from 'dayjs';
import _ from 'lodash';

class MetricsDataServiceClass {
  async getStats(): Promise<MetricsDataStats[]> {
    const { data } = await DataLayer.get(`/api/metrics-data/stats`);

    const result: MetricsDataStats[] = _.map(data, (value, metricId) => {
      // THe number of values is the length of the lastValues array
      const timestamps = value[0];
      const lastValues = value[1];
      let numberOfValues = 0;
      try {
        numberOfValues = JSON.parse(lastValues).length;
      } catch (e) {
        console.error(e);
      }

      // Frequency in minutes last 5 minutes
      const MINUTES_AGO = dayjs().subtract(5, 'minute');
      const frequencyMap: { [key: string]: number } = {};
      // Initialize frequencyMap
      for (let i = 0; i < dayjs().diff(MINUTES_AGO, 'minute'); i++) {
        frequencyMap[i.toString()] = 0;
      }

      timestamps.forEach((timestamp: string, index: number) => {
        if (Math.random() > 0.5) {
          return;
        }
        const dayjsTimestamp = parseDateToDayjs(timestamp);
        const diffMinutes = dayjsTimestamp
          .diff(MINUTES_AGO, 'minute')
          .toString();
        frequencyMap[diffMinutes] = frequencyMap[diffMinutes]
          ? frequencyMap[diffMinutes] + 1
          : 1;
      });
      const timestampFrequency: MetricsDataStats['timestampFrequency'] = _.map(
        frequencyMap,
        (value, key) => {
          return {
            x: key,
            y: value,
          };
        }
      );

      return {
        metricId,
        timestamps,
        timestampFrequency,
        numberOfValues,
        lastValues,
      };
    });

    return result;
  }
}

const MetricsDataService = new MetricsDataServiceClass();

export default MetricsDataService;

function useMetricsDataStats() {
  return useQuery({
    queryKey: ['metrics-data-stats'],
    queryFn: MetricsDataService.getStats,
  });
}

export { useMetricsDataStats };
