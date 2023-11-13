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
      return dayjs.unix(historyItem.created_at / 1000).format('YYYY-MM-DD');
    });

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
          return dayjs.unix(historyItem.created_at / 1000).format('HH');
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
                z: historyItems.some(
                  (historyItem) => historyItem.fail_message !== undefined
                )
                  ? 'Failed'
                  : 'Success',
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
        /* xInnerPadding={0.1}
        yInnerPadding={0.1} */
        margin={{ top: 50, right: 70, bottom: 30, left: 140 }}
        axisTop={{
          tickSize: 0,
          tickPadding: 10,
          // tickRotation: -90,
          /* legend: 'Time', */
          legendOffset: -40,
          legendPosition: 'middle',
        }}
        axisLeft={{
          tickSize: 0,
          tickPadding: 20,
          tickRotation: 0,
          format: (value) => {
            return value.replace(/-/g, '/');
          },
        }}
        colors={{
          type: 'diverging',
          divergeAt: 0.3,
          scheme: 'turbo',
          minValue: 0,
          maxValue,
        }}
        emptyColor="#EBEBEB"
        borderWidth={0}
        borderColor="#000000"
        enableLabels={true}
        labelTextColor={{
          from: 'color',
          modifiers: [['brighter', 3]],
        }}
        annotations={[]}
        tooltip={({ cell }) => {
          return cell.data.z === 'Failed' ? (
            <div className="bg-[#E0242E] p-2 text-white shadow-md">
              <div className="text-xs">
                Date: {cell.serieId.replace(/-/g, '/')}
              </div>
              <div className="text-xs">
                Time: {cell.data.x}:00~{cell.data.x}:59
              </div>
              <div className="text-xs">Activity: {cell.data.y}</div>
              {cell.data.z === 'Failed' ? (
                <div className="text-xs">Status: {cell.data.z}</div>
              ) : null}
            </div>
          ) : (
            <div className="bg-[#074eab] p-2 text-white shadow-md">
              <div className="text-xs">
                Date: {cell.serieId.replace(/-/g, '/')}
              </div>
              <div className="text-xs">
                Time: {cell.data.x}:00~{cell.data.x}:59
              </div>
              <div className="text-xs">Activity: {cell.data.y}</div>
              {cell.data.z === 'Failed' ? (
                <div className="text-xs">Status: {cell.data.z}</div>
              ) : null}
            </div>
          );
        }}
      />
    </div>
  );
}

export default memo(HistoryHeatmap);
