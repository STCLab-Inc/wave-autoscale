'use client';

import { ResponsiveHeatMapCanvas } from '@nivo/heatmap';
import dayjs, { Dayjs } from 'dayjs';
import { groupBy } from 'lodash';
import { memo, useMemo } from 'react';

interface HistoryHeatmapProps {
  history: any;
  from: Dayjs;
  to: Dayjs;
}
function HistoryHeatmap({ history, from, to }: HistoryHeatmapProps) {
  const dataForHeatmap = useMemo(() => {
    if (!history) {
      return [];
    }
    const grouped = groupBy(history, (historyItem) => {
      return dayjs.unix(historyItem.created_at).format('YYYY-MM-DD');
    });

    let current = from;
    while (current.isBefore(to)) {
      const date = current.format('YYYY-MM-DD');
      if (!grouped[date]) {
        grouped[date] = [];
      }
      current = current.add(1, 'day');
    }

    const data = Object.entries(grouped)
      .sort((a, b) => (a[0] < b[0] ? 0 : 1))
      .map(([date, historyItems]) => {
        const groupedByHour = groupBy(historyItems, (historyItem) => {
          return dayjs.unix(historyItem.created_at).format('HH');
        });

        for (let i = 0; i < 24; i++) {
          const hour = i.toString().padStart(2, '0');
          if (!groupedByHour[hour]) {
            groupedByHour[hour] = [];
          }
        }

        return {
          id: date,
          data: Object.entries(groupedByHour)
            .sort((a, b) => (a[0] < b[0] ? 0 : 1))
            .map(([hour, historyItems]) => {
              return {
                x: hour,
                y: historyItems.length,
              };
            }),
        };
      });
    return data;
  }, [history, from, to]);

  console.dir({ dataForHeatmap }, { depth: null });
  return (
    <div className="h-80 w-full">
      <ResponsiveHeatMapCanvas
        data={dataForHeatmap}
        margin={{ top: 70, right: 60, bottom: 20, left: 80 }}
        axisTop={{
          tickSize: 5,
          tickPadding: 5,
          tickRotation: -90,
          legend: '',
          legendOffset: 46,
        }}
        axisRight={{
          tickSize: 5,
          tickPadding: 5,
          tickRotation: 0,
          legend: 'Date',
          legendPosition: 'middle',
          legendOffset: 40,
        }}
        axisLeft={null}
        colors={{
          type: 'sequential',
          scheme: 'red_yellow_blue',
          minValue: 0,
          maxValue: 1,
        }}
        emptyColor="#555555"
        borderWidth={1}
        borderColor="#000000"
        enableLabels={true}
        legends={[
          {
            anchor: 'left',
            translateX: -50,
            translateY: 0,
            length: 200,
            thickness: 10,
            direction: 'column',
            tickPosition: 'after',
            tickSize: 3,
            tickSpacing: 4,
            tickOverlap: false,
            tickFormat: '>-.2s',
            title: 'Value →',
            titleAlign: 'start',
            titleOffset: 4,
          },
        ]}
        annotations={[]}
      />
    </div>
  );
}

export default memo(HistoryHeatmap);
