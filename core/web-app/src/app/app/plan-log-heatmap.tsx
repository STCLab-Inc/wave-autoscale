import { memo, useMemo } from 'react';
import dayjs, { Dayjs } from 'dayjs';
import { groupBy } from 'lodash';
import { ResponsiveHeatMap, ResponsiveHeatMapCanvas } from '@nivo/heatmap';
import { PlanLogDefinitionEx } from '../../types/plan-log-definition-ex';
import { addAlpha } from '@/utils/color';
import { parseDateToDayjs } from '@/utils/date';

const MAX_SUCCESS_COUNT = 20;

// Interfaces

interface PlanLogDefinitionExWithDayjs extends PlanLogDefinitionEx {
  created_at_dayjs?: Dayjs;
}

interface PlanLogHeatmapProps {
  planLogs: PlanLogDefinitionExWithDayjs[];
  from: Dayjs;
  to: Dayjs;
}

function PlanLogHeatmap({ planLogs, from, to }: PlanLogHeatmapProps) {
  const dataForHeatmap = useMemo(() => {
    if (!planLogs) {
      return [];
    }

    const groupedByDate = groupBy(planLogs, (planLog) => {
      // Add dayjs to the planLog to avoid parsing the date multiple times
      planLog.created_at_dayjs = parseDateToDayjs(planLog.created_at);
      return planLog.created_at_dayjs?.format('YYYY-MM-DD');
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
      .sort(([dateA], [dateB]) => (dateA < dateB ? -1 : 1))
      .map(([date, planLogsByDate]) => {
        const groupedByHour = groupBy(planLogsByDate, (planLog) =>
          planLog.created_at_dayjs?.format('HH')
        );

        for (let i = 0; i < 24; i++) {
          const hour = i.toString().padStart(2, '0');
          if (!groupedByHour[hour]) {
            groupedByHour[hour] = [];
          }
        }

        return {
          // Axis Left
          id: date,
          // Axis Top
          data: Object.entries(groupedByHour)
            .sort(([hourA], [hourB]) => (hourA < hourB ? -1 : 1))
            .map(([hour, planLogs]) => {
              const totalCount = planLogs.length;
              const errorCount = planLogs.filter(
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
  }, [planLogs, from, to]);

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
    <div className="h-full w-full">
      <ResponsiveHeatMap
        data={dataForHeatmap}
        margin={{ top: 0, right: 0, bottom: 0, left: 40 }}
        axisTop={null}
        axisLeft={{
          tickSize: 0,
          tickPadding: 20,
          tickRotation: 0,
          format: (date) => dayjs(date).format('ddd'),
        }}
        borderRadius={5}
        forceSquare
        colors={(cell) => {
          // console.log({ cell });
          const { data } = cell;
          // // Error
          // if (data.errorCount > 0) {
          //   return addAlpha('#E0242E', data.errorCount / MAX_ERROR_COUNT);
          // }
          if (data.y > 0) {
            return addAlpha('#863EBE', data.y / MAX_SUCCESS_COUNT);
          }
          return '#EDEEF1';
        }}
        xInnerPadding={0.2}
        yInnerPadding={0.2}
        emptyColor="#EBEBEB"
        borderWidth={0}
        borderColor="#000000"
        enableLabels={false}
        // labelTextColor={(cell) => {
        //   const { data } = cell;
        //   // Error
        //   if (data.errorCount > 0) {
        //     return '#FFFFFF';
        //   }
        //   if (data.y > 0) {
        //     return '#FFFFFF';
        //   }
        //   return '#00000000';
        // }}
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

export default memo(PlanLogHeatmap);
