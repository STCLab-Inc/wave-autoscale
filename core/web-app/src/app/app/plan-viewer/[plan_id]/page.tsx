'use client';

import React, { useEffect, useMemo, useState } from 'react';
import PageHeader from '../../common/page-header';
import { createColumnHelper } from '@tanstack/react-table';
import { usePlanLogs } from '@/services/plan-log';
import dayjs, { Dayjs } from 'dayjs';
import WAVirtualizedTable from '../../common/wa-virtualized-table';
import { PlanLogDefinitionEx } from '../../../../types/plan-log-definition-ex';
import StatusBadge from './status-badge';
import { renderKeyValuePairsWithJson } from '../../common/keyvalue-renderer';
import { reverse } from 'lodash';
import { usePathname, useRouter, useSearchParams } from 'next/navigation';
import AutoscalingTimelineChart from './autoscaling-timeline-chart';
import { useScalingPlan } from '@/services/scaling-plan';
import { extractMetricsFromScalingPlan } from '@/utils/scaling-plan';
import MetricsDataService from '@/services/metrics-data';
import { GetJSParamDefinition } from '@/types/get-js-param-definition';
import { MetricsData, MetricsDataItem } from '@/types/metrics-data-stats';
import WAAreaChart from '../../common/wa-area-chart';
import { parseDateToDayjs } from '@/utils/date';
import MetricsTimelineChart from './metrics-timeline-chart';

// Default Values
const DEFAULT_FROM = dayjs().subtract(3, 'hours');
const DEFAULT_TO = dayjs();
const DEFAULT_DATE_FORMAT = 'MM-DD HH:mm';

const formatDateTime = (date: Dayjs) => date.format('YYYY-MM-DDTHH:mm');
const parseDateTime = (date: string) => dayjs(date, 'YYYY-MM-DDTHH:mm');
// Table columns
const columnHelper = createColumnHelper<PlanLogDefinitionEx>();
const columns = [
  // // Index
  // columnHelper.display({
  //   id: 'index',
  //   header: () => '#',
  //   size: 50,
  //   cell: (cell) => cell.row.index + 1,
  // }),
  columnHelper.accessor('created_at', {
    header: () => 'Date',
    cell: (cell) => {
      const date = cell.getValue();
      return dayjs.unix(date).format('YYYY-MM-DD HH:mm:ss');
    },
  }),
  columnHelper.accessor('metric_values_json', {
    header: () => 'Metrics',
    cell: (cell) => {
      let json: any[] = [];
      try {
        json = JSON.parse(cell.getValue());
      } catch (error) {
        // console.error('Failed to parse json', { error });
      }

      return (
        <div>
          {json?.map((item, index) => {
            return (
              <div key={index}>{renderKeyValuePairsWithJson(item, true)}</div>
            );
          })}
        </div>
      );
    },
  }),
  columnHelper.accessor('metadata_values_json', {
    header: () => 'Scaling Components',
    cell: (cell) => {
      return renderKeyValuePairsWithJson(cell.getValue(), true);
    },
  }),
  columnHelper.display({
    id: 'status',
    header: () => 'Status',
    cell: (cell) => {
      const failMessage = cell.row.original.fail_message;
      return (
        <div>
          <StatusBadge success={!failMessage} />
          {failMessage && (
            <div className="mt-2 whitespace-pre text-xs">{failMessage}</div>
          )}
        </div>
      );
    },
  }),
];

type MetricsDataItemEx = MetricsDataItem & { metricId: string };

export default function PlanViewerDetailPage({
  params,
}: {
  params: { plan_id: string };
}) {
  const router = useRouter();
  const pathname = usePathname();
  // Data
  const planId = params.plan_id;
  const searchParams = useSearchParams();
  const from = searchParams.get('from') || formatDateTime(DEFAULT_FROM);
  const to = searchParams.get('to') || formatDateTime(DEFAULT_TO);

  const [metricsData, setMetricsData] = useState<MetricsDataItemEx[]>([]);
  const { data: scalingPlan } = useScalingPlan(planId);
  const { data: autoscalingLogs, isLoading } = usePlanLogs(
    planId,
    dayjs(from).startOf('day'),
    dayjs(to).endOf('day')
  );
  const autoscalingLogsForGraph = useMemo(() => {
    return autoscalingLogs?.map((log) => {
      return {
        ...log,
        status: log.fail_message ? false : true,
        minutes: Math.floor(dayjs.unix(log.created_at).minute() / 10),
      };
    });
  }, [autoscalingLogs]);
  const reversedData = reverse([...(autoscalingLogs ?? [])]);

  // Effects
  useEffect(() => {
    if (!scalingPlan) {
      return;
    }
    const task = async () => {
      const metricsInfo = extractMetricsFromScalingPlan(scalingPlan);
      const metricsData = await Promise.all(
        metricsInfo.map(async (getJsParam) => {
          const data = await MetricsDataService.getMetricsData(
            getJsParam.metricId,
            parseDateTime(from),
            parseDateTime(to)
          );
          const metrics = data.find(
            (metricsDataItem: MetricsDataItem, index: number) => {
              if (
                metricsDataItem.name &&
                getJsParam.name &&
                metricsDataItem.name !== getJsParam.name
              ) {
                return false;
              }
              if (metricsDataItem.tags && getJsParam.tags) {
                if (
                  Object.keys(getJsParam.tags).length !==
                  Object.keys(metricsDataItem.tags).length
                ) {
                  return false;
                }
                for (const key in getJsParam.tags) {
                  if (metricsDataItem.tags[key] !== getJsParam.tags[key]) {
                    return false;
                  }
                }
              }
              return true;
            }
          );
          if (!metrics) {
            return;
          }
          (metrics as MetricsDataItemEx).metricId = getJsParam.metricId;
          return metrics;
        })
      );
      const filtered = metricsData.filter(
        (item) => item !== undefined
      ) as MetricsDataItemEx[];
      setMetricsData(filtered);
    };

    task();
  }, [scalingPlan, from, to]);

  // Handlers
  // Date Picker Handler
  const handleDate = (field: 'from' | 'to', value: string) => {
    const newDate = dayjs(value);
    const isValidDate = newDate.isValid();

    if (!isValidDate) {
      return;
    }
    const params = {
      from: field === 'from' ? formatDateTime(newDate) : from,
      to: field === 'to' ? formatDateTime(newDate) : to,
    };
    router.push(`${pathname}?from=${params.from}&to=${params.to}`);
  };

  console.log({ metricsData });

  return (
    <main className="flex h-full w-full flex-col">
      {/* Page Header */}
      <PageHeader title="Plan Viewer" backButton subtitle={planId} />
      <div className="flex flex-1 flex-col py-6">
        {/* Controls */}
        <div className="flex space-x-4 px-6">
          <div className="flex items-center">
            <div className="form-control mr-2">
              <label className="input-group-sm">
                <input
                  type="datetime-local"
                  className="input-bordered input input-sm max-w-[200px] cursor-text px-2 text-center focus:outline-none"
                  max={formatDateTime(dayjs(to))}
                  value={from}
                  onChange={(event) => handleDate('from', event.target.value)}
                />
              </label>
            </div>
            <span>~</span>
            <div className="form-control ml-2">
              <label className="input-group-sm">
                <input
                  type="datetime-local"
                  className="input-bordered input input-sm max-w-[200px] cursor-text px-2 text-center focus:outline-none"
                  min={formatDateTime(dayjs(from))}
                  value={to}
                  onChange={(event) => handleDate('to', event.target.value)}
                />
              </label>
            </div>
          </div>
        </div>
        {/* Composite Graph */}
        <div className="flex flex-col p-6">
          <div className="wa-card flex-1 flex-col px-10 py-10">
            {metricsData.map((item, index) => {
              return (
                <div key={index} className="flex items-center">
                  <div className="font-bold">{item.metricId}</div>
                  <div className="h-[100px] flex-1">
                    <MetricsTimelineChart
                      data={item.values}
                      yDataKey="value"
                      xDataKey="timestamp"
                      simpleXAxis
                      autoscalingLogs={autoscalingLogsForGraph}
                      xTickFormatter={(value) => {
                        const date = parseDateToDayjs(value);
                        if (!date) {
                          return '';
                        }
                        return date.format(DEFAULT_DATE_FORMAT);
                      }}
                    />
                  </div>
                </div>
              );
            })}
            {metricsData.map((item, index) => {
              return (
                <div key={index} className="flex items-center">
                  <div className="font-bold">{item.metricId}</div>
                  <div key={index} className="h-[100px] flex-1">
                    <MetricsTimelineChart
                      data={item.values}
                      yDataKey="value"
                      xDataKey="timestamp"
                      autoscalingLogs={autoscalingLogsForGraph}
                      xTickFormatter={(value) => {
                        const date = parseDateToDayjs(value);
                        if (!date) {
                          return '';
                        }
                        return date.format(DEFAULT_DATE_FORMAT);
                      }}
                    />
                  </div>
                </div>
              );
            })}
            {/* <div className="h-[100px]">
              <AutoscalingTimelineChart
                data={autoscalingLogsForGraph}
                xDataKey="created_at"
                yDataKey="minutes"
                zDataKey="status"
                xFrom={dayjs(from).unix()}
                xTo={dayjs(to).unix()}
              />
            </div> */}
          </div>
        </div>
        {/* Table */}
        <div className="flex flex-1 flex-col p-6">
          <div className="wa-card flex-1">
            <WAVirtualizedTable<PlanLogDefinitionEx>
              tableOptions={{
                data: reversedData ?? [],
                columns,
              }}
              isLoading={isLoading}
              onRowClick={(row) => {
                // setSelectedRow(row);
                // (document.getElementById(DETAIL_MODAL_ID) as any).showModal();
              }}
            />
          </div>
        </div>
      </div>
    </main>
  );
}
