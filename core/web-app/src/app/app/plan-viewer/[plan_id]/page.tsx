'use client';

import React, { useEffect, useMemo, useState } from 'react';
import PageHeader from '../../common/page-header';
import { createColumnHelper } from '@tanstack/react-table';
import { usePlanLogs } from '@/services/plan-log';
import dayjs, { Dayjs } from 'dayjs';
import WAVirtualizedTable from '../../common/wa-virtualized-table';
import StatusBadge from './status-badge';
import { renderKeyValuePairsWithJson } from '../../common/keyvalue-renderer';
import { reverse } from 'lodash';
import { usePathname, useRouter, useSearchParams } from 'next/navigation';
import { useScalingPlan } from '@/services/scaling-plan';
import { extractMetricsFromScalingPlan } from '@/utils/scaling-plan';
import MetricsDataService from '@/services/metrics-data';
import { MetricsDataItem } from '@/types/metrics-data-stats';
import { parseDateToDayjs } from '@/utils/date';
import MetricsTimelineChart from './metrics-timeline-chart';
import { PlanLogDefinitionEx } from '@/types/plan-log-definition-ex';
import classNames from 'classnames';

// Default Values
const DEFAULT_FROM = dayjs().subtract(1, 'hours');
const DEFAULT_TO = dayjs();
const DEFAULT_DATE_FORMAT = 'MM-DD HH:mm';

const formatDateTime = (date: Dayjs) => date.format('YYYY-MM-DDTHH:mm');
const parseDateTime = (date: string) => dayjs(date, 'YYYY-MM-DDTHH:mm');
// Table columns
const columnHelper = createColumnHelper<PlanLogDefinitionEx>();
const columns = [
  columnHelper.accessor('created_at', {
    header: () => 'Date',
    cell: (cell) => {
      const date = cell.getValue();
      return parseDateToDayjs(date)?.format('YYYY-MM-DD HH:mm:ss');
    },
    size: 200,
  }),
  columnHelper.accessor('metric_values_json', {
    id: 'metrics',
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
    enableResizing: true,
    minSize: 0,
    size: 0,
  }),
  columnHelper.accessor('metadata_values_json', {
    id: 'metadata',
    header: () => 'Scaling Components',
    cell: (cell) => {
      return renderKeyValuePairsWithJson(cell.getValue(), true);
    },
    enableResizing: true,
    minSize: 0,
    size: 0,
  }),
  columnHelper.display({
    id: 'status',
    header: () => 'Status',
    cell: (cell) => {
      const failMessage = cell.row.original.fail_message;
      return (
        <div>
          <StatusBadge success={!failMessage} />
          {failMessage && <div className="mt-2 text-xs">{failMessage}</div>}
        </div>
      );
    },
    size: 200,
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

  // From, To
  const from = searchParams.get('from') || formatDateTime(DEFAULT_FROM);
  const to = searchParams.get('to') || formatDateTime(DEFAULT_TO);
  const fromDayjs = useMemo(() => dayjs(from).startOf('minute'), [from]);
  const toDayjs = useMemo(() => dayjs(to).endOf('minute'), [to]);

  const [metricsData, setMetricsData] = useState<MetricsDataItemEx[]>([]);
  // Get the scaling plan
  const { data: scalingPlan } = useScalingPlan(planId);

  // Get the plan logs regarding this plan and transform them to the format that the graph can use
  const { data: planLogs, isLoading } = usePlanLogs(planId, fromDayjs, toDayjs);
  const planLogsForGraph = useMemo(() => {
    return planLogs?.map((log) => {
      return {
        ...log,
        timestamp: log.created_at,
        status: log.fail_message ? false : true,
        minutes: Math.floor(dayjs.unix(log.created_at).minute() / 10),
      };
    });
  }, [planLogs]);

  // Stats for the plan logs
  const totalCount = useMemo(() => planLogs?.length ?? 0, [planLogs]);
  const successCount = useMemo(() => {
    return planLogs?.filter((log) => !log.fail_message).length ?? 0;
  }, [planLogs]);
  const failCount = useMemo(
    () => totalCount - successCount,
    [totalCount, successCount]
  );
  const reversedData = useMemo(
    () => reverse([...(planLogs ?? [])]),
    [planLogs]
  );

  // Effects

  // Get the metrics data for the scaling plan
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
            fromDayjs,
            toDayjs
          );

          // Filter the metrics data
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

          // Add metricId to the metrics data
          (metrics as MetricsDataItemEx).metricId = getJsParam.metricId;
          metrics.values = metrics.values.map((item) => {
            return {
              ...item,
              timestamp: parseDateToDayjs(item.timestamp)?.valueOf() ?? 0,
            };
          });
          return metrics;
        })
      );

      // Filter the metrics data that is not found
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
    let newDate = dayjs(value);
    const isValidDate = newDate.isValid();

    if (!isValidDate) {
      return;
    }

    if (field === 'from' && newDate.isAfter(parseDateTime(to))) {
      newDate = parseDateTime(to);
    } else if (field === 'to' && newDate.isBefore(parseDateTime(from))) {
      newDate = parseDateTime(from);
    }
    const params = {
      from: field === 'from' ? formatDateTime(newDate) : from,
      to: field === 'to' ? formatDateTime(newDate) : to,
    };
    router.push(`${pathname}?from=${params.from}&to=${params.to}`);
  };

  const handleRefreshWithMinutes = (minutes: number) => {
    const newDate = dayjs().subtract(minutes, 'minutes');
    const params = {
      from: formatDateTime(newDate),
      to: formatDateTime(dayjs()),
    };
    router.push(`${pathname}?from=${params.from}&to=${params.to}`);
  };

  return (
    <main className="flex h-full w-full flex-col">
      {/* Page Header */}
      <PageHeader
        title="Plan Viewer"
        backButton
        subtitle={planId}
        backUrl="/app/plan-viewer"
      />
      <div className="flex flex-1 flex-col py-3 pb-6">
        {/* Controls */}
        <div className="flex justify-between space-x-4 px-6">
          {/* From, To */}
          <div className="flex flex-1 items-center">
            <div className="form-control mr-2">
              <div className="label">
                <span className="label-text text-xs font-bold">From</span>
              </div>
              <label className="input-group-sm">
                <input
                  type="datetime-local"
                  className="input-bordered input input-sm w-[200px] cursor-text px-2 text-center focus:outline-none"
                  max={formatDateTime(dayjs(to))}
                  value={from}
                  onChange={(event) => handleDate('from', event.target.value)}
                />
              </label>
            </div>
            <div className="form-control">
              <div className="label">
                <span className="label-text text-xs font-bold">To</span>
              </div>
              <label className="input-group-sm">
                <input
                  type="datetime-local"
                  className="input-bordered input input-sm w-[200px] cursor-text px-2 text-center focus:outline-none"
                  min={formatDateTime(dayjs(from))}
                  value={to}
                  onChange={(event) => handleDate('to', event.target.value)}
                />
              </label>
            </div>
            <div className="form-control ml-4">
              <div className="label">
                <span className="label-text text-xs font-bold">Quick</span>
              </div>
              <div className="join">
                <button
                  className="btn btn-xs join-item"
                  onClick={() => handleRefreshWithMinutes(5)}
                >
                  Last 5 minutes
                </button>
                <button
                  className="btn btn-xs join-item"
                  onClick={() => handleRefreshWithMinutes(30)}
                >
                  Last 30 minutes
                </button>
                <button
                  className="btn btn-xs join-item"
                  onClick={() => handleRefreshWithMinutes(60)}
                >
                  Last 1 hour
                </button>
              </div>
            </div>
          </div>
          {/* Stats */}
          {!isLoading && (
            <div className="form-control">
              <div className="label">
                <span className="label-text text-xs font-bold">Stats</span>
              </div>
              <div className="flex h-8 items-center gap-2">
                <div className="badge badge-primary gap-2 px-5 py-3 text-xs">
                  Total {totalCount}
                </div>
                <div className="badge badge-success gap-2 px-5 py-3 text-xs">
                  Success {successCount}
                </div>
                <div className="badge badge-error gap-2 px-5 py-3 text-xs text-white">
                  Fail {failCount}
                </div>
              </div>
            </div>
          )}
        </div>
        {/* Composite Graph */}
        <div
          className={classNames(
            'flex min-h-[150px] flex-col p-6',
            `h-[${metricsData.length * 100 + 100}px]`
          )}
        >
          <div className="wa-card flex-col px-10 py-10">
            {isLoading && <div className="skeleton h-full" />}
            {!isLoading && metricsData.length === 0 && (
              <div className="text-wa-gray-500 flex h-full items-center justify-center text-2xl text-gray-200">
                No metrics data
              </div>
            )}
            {!isLoading &&
              metricsData.map((item, index) => {
                return (
                  <div key={index} className="flex items-center">
                    <div className="w-40 overflow-hidden break-all font-bold">
                      {item.metricId}
                    </div>
                    <div key={index} className="h-[100px] flex-1">
                      <MetricsTimelineChart
                        data={item.values}
                        yDataKey="value"
                        xDataKey="timestamp"
                        planLogs={planLogsForGraph}
                        simpleXAxis={index !== metricsData.length - 1}
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
              rowHeight={70}
              isLoading={isLoading}
            />
          </div>
        </div>
      </div>
    </main>
  );
}
