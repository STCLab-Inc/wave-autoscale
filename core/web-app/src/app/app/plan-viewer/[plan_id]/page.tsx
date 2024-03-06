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
const DEFAULT_FROM = dayjs().subtract(3, 'hours');
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
  }),
  columnHelper.accessor('metadata_values_json', {
    id: 'metadata',
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
          {failMessage && <div className="mt-2 text-xs">{failMessage}</div>}
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
  const { data: planLogs, isLoading } = usePlanLogs(
    planId,
    dayjs(from),
    dayjs(to)
  );
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
          metrics.values = metrics.values.map((item) => {
            return {
              ...item,
              timestamp: parseDateToDayjs(item.timestamp)?.valueOf() ?? 0,
            };
          });
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

  return (
    <main className="flex h-full w-full flex-col">
      {/* Page Header */}
      <PageHeader
        title="Plan Viewer"
        backButton
        subtitle={planId}
        backUrl="/app/plan-viewer"
      />
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
          {/* Stats */}
          {!isLoading && (
            <div className="flex flex-1 gap-4 text-sm">
              <div className="flex items-center gap-2">
                <span className="text-wa-gray-500">Total:</span>
                <span className="text-wa-gray-700">{totalCount} logs</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-wa-gray-500">Success:</span>
                <span className="text-wa-gray-700">{successCount} logs</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-wa-gray-500">Fail:</span>
                <span className="text-wa-gray-700">{failCount} logs</span>
              </div>
            </div>
          )}
        </div>
        {/* Composite Graph */}
        <div
          className={classNames(
            'flex flex-col p-6',
            `h-[${metricsData.length * 100 + 100}px]`
          )}
        >
          <div className="wa-card flex-col px-10 py-10">
            {isLoading && <div className="skeleton h-[100px]" />}
            {!isLoading && metricsData.length === 0 && (
              <div className="text-wa-gray-500 h-[100px] text-center text-2xl text-gray-200">
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
              rowHeight={100}
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
