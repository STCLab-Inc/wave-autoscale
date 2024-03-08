import { memo, useMemo } from 'react';
import dayjs, { Dayjs } from 'dayjs';
import { groupBy } from 'lodash';
import { ResponsiveHeatMapCanvas } from '@nivo/heatmap';
import { PlanLogDefinitionEx } from '../../../types/plan-log-definition-ex';
import { addAlpha } from '@/utils/color';

const MAX_ERROR_COUNT = 10;
const MAX_SUCCESS_COUNT = 10;

// Interfaces
interface AutoscalingHistoryHeatmapProps {
  autoscalingHistory: PlanLogDefinitionEx[];
  from: Dayjs;
  to: Dayjs;
}

function AutoscalingHistoryHeatmap({
  autoscalingHistory,
  from,
  to,
}: AutoscalingHistoryHeatmapProps) {
  const dataForHeatmap = useMemo(() => {
    if (!autoscalingHistory) {
      return [];
    }

    const groupedByDate = groupBy(
      autoscalingHistory,
      (autoscalingHistoryItem) =>
        dayjs
          .unix(autoscalingHistoryItem.created_at / 1000)
          .format('YYYY-MM-DD')
    );

    let current = from;
    while (current.isBefore(to) || current.isSame(to)) {
      const date = current.format('YYYY-MM-DD');
      if (!groupedByDate[date]) {
        groupedByDate[date] = [];
      }
      current = current.add(1, 'day');
    }

    const data = Object.entries(groupedByDate)
      .sort(([dateA], [dateB]) => (dateA < dateB ? -1 : 1))
      .map(([date, autoscalingHistoryItems]) => {
        const groupedByHour = groupBy(
          autoscalingHistoryItems,
          (autoscalingHistoryItem) =>
            dayjs.unix(autoscalingHistoryItem.created_at / 1000).format('HH')
        );

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
            .map(([hour, autoscalingHistoryItems]) => {
              const totalCount = autoscalingHistoryItems.length;
              const errorCount = autoscalingHistoryItems.filter(
                (autoscalingHistoryItem) => autoscalingHistoryItem.fail_message
              ).length;
              return {
                x: hour,
                y: totalCount,
                // Additional information
                successCount: totalCount - errorCount,
                errorCount,
              };
            }),
        };
      });

    return data;
  }, [autoscalingHistory, from, to]);

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
        data={dataForHeatmap}
        margin={{ top: 50, right: 70, bottom: 30, left: 140 }}
        axisTop={{
          tickSize: 0,
          tickPadding: 10,
          legendOffset: -40,
          legendPosition: 'middle',
        }}
        axisLeft={{
          tickSize: 0,
          tickPadding: 20,
          tickRotation: 0,
          format: (value) => value.replace(/-/g, '/'),
        }}
        colors={(cell) => {
          console.log({ cell });
          const { data } = cell;
          // Error
          if (data.errorCount > 0) {
            return addAlpha('#E0242E', data.errorCount / MAX_ERROR_COUNT);
          }
          if (data.y > 0) {
            return addAlpha('#09AB6E', data.y / MAX_SUCCESS_COUNT);
          }
          return '#EDEEF1';
        }}
        xInnerPadding={0.2}
        yInnerPadding={0.2}
        emptyColor="#EBEBEB"
        borderWidth={0}
        borderColor="#000000"
        enableLabels={true}
        labelTextColor={(cell) => {
          const { data } = cell;
          // Error
          if (data.errorCount > 0) {
            return '#FFFFFF';
          }
          if (data.y > 0) {
            return '#FFFFFF';
          }
          return '#00000000';
        }}
        annotations={[]}
        tooltip={({ cell }) => (
          <div
            className={`p-2 text-white shadow-md ${
              cell.data.errorCount > 0 ? 'bg-[#E0242E]' : 'bg-[#074EAB]'
            }`}
          >
            <div className="text-xs">
              Date: {cell.serieId.replace(/-/g, '/')}
            </div>
            <div className="text-xs">
              Time: {cell.data.x}:00~{cell.data.x}:59
            </div>
            <div className="text-xs">Total: {cell.data.y}</div>
            {cell.data.successCount > 0 ? (
              <div className="text-xs">Success: {cell.data.successCount}</div>
            ) : null}
            {cell.data.errorCount > 0 ? (
              <div className="text-xs">Fail: {cell.data.errorCount}</div>
            ) : null}
          </div>
        )}
      />
    </div>
  );
}

export default memo(AutoscalingHistoryHeatmap);
