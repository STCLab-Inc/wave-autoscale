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
    const groupedByDate = groupBy(history, (historyItem) => {
      return dayjs.unix(historyItem.created_at).format('YYYY-MM-DD');
    });

    // Fill in missing dates
    let current = from;
    while (current.isBefore(to) || current.isSame(to)) {
      const date = current.format('YYYY-MM-DD');
      if (!groupedByDate[date]) {
        groupedByDate[date] = [];
      }
      current = current.add(1, 'day');
    }

    const data = Object.entries(groupedByDate)
      .sort(([dateA], [dateB]) => {
        return dateA < dateB ? -1 : 1;
      })
      .map(([date, historyItems]) => {
        const groupedByHour = groupBy(historyItems, (historyItem) => {
          return dayjs.unix(historyItem.created_at).format('HH');
        });

        // Fill in missing hours
        for (let i = 0; i < 24; i++) {
          const hour = i.toString().padStart(2, '0');
          if (!groupedByHour[hour]) {
            groupedByHour[hour] = [];
          }
        }

        return {
          id: date,
          data: Object.entries(groupedByHour)
            .sort(([hourA], [hourB]) => (hourA < hourB ? -1 : 1))
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

  const maxValue = useMemo(() => {
    let maxValue = 0;
    dataForHeatmap.forEach((dataByDate) => {
      dataByDate.data.forEach((dataByHour) => {
        if (dataByHour.y > maxValue) {
          maxValue = dataByHour.y;
        }
      });
    });
    return maxValue || 1;
  }, [dataForHeatmap]);

  return (
    <div className="h-80 w-full">
      <ResponsiveHeatMapCanvas
        // forceSquare={true}
        data={dataForHeatmap}
        xInnerPadding={0.1}
        yInnerPadding={0.1}
        margin={{ top: 70, right: 80, bottom: 20, left: 120 }}
        axisTop={{
          tickSize: 0,
          tickPadding: 10,
          // tickRotation: -90,
          legend: 'Time',
          legendOffset: -40,
          legendPosition: 'middle',
        }}
        axisLeft={{
          tickSize: 0,
          tickPadding: 20,
          tickRotation: 0,
        }}
        colors={{
          type: 'diverging',
          divergeAt: 0.4,
          scheme: 'greens',
          minValue: 0,
          maxValue,
        }}
        emptyColor="#EBEBEB"
        borderWidth={0}
        borderColor="#000000"
        enableLabels={true}
        labelTextColor={{
          from: 'color',
          modifiers: [['brighter', 2]],
        }}
        annotations={[]}
      />
    </div>
  );
}

export default memo(HistoryHeatmap);
