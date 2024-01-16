'use client';

import React, { useEffect, useMemo, useState } from 'react';
import { useSearchParams, useRouter, usePathname } from 'next/navigation';
import dayjs, { Dayjs } from 'dayjs';
import { decodeTime } from 'ulid';
import AutoscalingHistoryService from '@/services/autoscaling-history';
import { AutoscalingHistoryDefinition } from '@/types/bindings/autoscaling-history-definition';
import AutoscalingHistoryHeatmap from './autoscaling-history-heatmap';
import { AutoscalingHistoryDefinitionEx } from './autoscaling-history-definition-ex';
import PageHeader from '../common/page-header';
import WASimpleTable from '../common/wa-simple-table';
import { createColumnHelper } from '@tanstack/react-table';
import StatusBadge from './status-badge';
import { renderKeyValuePairsWithJson } from '../common/keyvalue-renderer';

// Default Values
const DEFAULT_FROM = dayjs().subtract(7, 'days');
const DEFAULT_TO = dayjs();

// Table columns
const columnHelper = createColumnHelper<AutoscalingHistoryDefinitionEx>();
const columns = [
  // Index
  columnHelper.display({
    id: 'index',
    header: () => '#',
    size: 50,
    cell: (cell) => cell.row.index + 1,
  }),
  columnHelper.accessor('plan_id', {
    header: () => 'Scaling Plan ID',
    cell: (cell) => cell.getValue(),
  }),
  columnHelper.accessor('metric_values_json', {
    header: () => 'Metrics',
    cell: (cell) => {
      let json: any[] = [];
      try {
        json = JSON.parse(cell.getValue());
      } catch (error) {
        console.error('Failed to parse json', { error });
      }

      return (
        <div>
          {json?.map((item, index) => {
            return renderKeyValuePairsWithJson(item, true);
          })}
        </div>
      );
    },
  }),
  columnHelper.accessor('metadata_values_json', {
    header: () => 'Metadata',
    cell: (cell) => {
      return renderKeyValuePairsWithJson(cell.getValue(), true);
    },
  }),
  columnHelper.display({
    id: 'status',
    header: () => 'Status',
    cell: (cell) => {
      const failMessage = cell.row.original.fail_message;
      if (failMessage) {
        console.log(failMessage);
      }
      return (
        <div>
          <StatusBadge success={!failMessage} />
          {failMessage && <div className="mt-2 text-xs">{failMessage}</div>}
        </div>
      );
    },
  }),
  columnHelper.accessor('created_at', {
    header: () => 'Date',
    cell: (cell) => {
      const date = cell.getValue();
      return dayjs(date).format('YYYY-MM-DD HH:mm:ss');
    },
  }),
];

// Utils
const formatDate = (date: Dayjs) => date.format('YYYY-MM-DD');

async function getAutoscalingHistory(from: Dayjs, to: Dayjs) {
  const autoscalingHistory =
    await AutoscalingHistoryService.getAutoscalingHistoryByFromTo(from, to);
  return autoscalingHistory;
}

export default function AutoscalingHistoryPage() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  const from = searchParams.get('from') || formatDate(DEFAULT_FROM);
  const to = searchParams.get('to') || formatDate(DEFAULT_TO);
  const fromDayjs = useMemo(() => dayjs(from).startOf('day'), [from]);
  const toDayjs = useMemo(() => dayjs(to).endOf('day'), [to]);

  // UI
  const [isFetching, setIsFetching] = useState(false);

  // Data
  const [autoscalingHistory, setAutoscalingHistory] = useState<
    AutoscalingHistoryDefinitionEx[]
  >([]);

  // Effects

  // Fetch Autoscaling History Data when the page is loaded with from and to params
  useEffect(() => {
    fetchAutoscalingHistory();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [fromDayjs, toDayjs]);

  // Handlers

  // Fetch Autoscaling History Data
  const fetchAutoscalingHistory = async () => {
    setIsFetching(true);
    try {
      let autoscalingHistoryData = await getAutoscalingHistory(
        fromDayjs,
        toDayjs
      );
      autoscalingHistoryData = autoscalingHistoryData.map(
        (autoscalingHistoryDataItem: AutoscalingHistoryDefinition) =>
          ({
            ...autoscalingHistoryDataItem,
            created_at: decodeTime(autoscalingHistoryDataItem.id),
          } as AutoscalingHistoryDefinitionEx)
      );
      setAutoscalingHistory(autoscalingHistoryData);
    } catch (error) {
      console.error({ error });
    }
    setIsFetching(false);
  };
  // Date Picker Handler
  const handleDate = (field: 'from' | 'to', value: string) => {
    const newDate = dayjs(value);
    const isValidDate = newDate.isValid();

    if (!isValidDate) {
      return;
    }
    const params = {
      from: field === 'from' ? formatDate(newDate) : from,
      to: field === 'to' ? formatDate(newDate) : to,
    };
    router.push(`${pathname}?from=${params.from}&to=${params.to}`);
  };

  return (
    <main className="flex h-full w-full flex-col">
      {/* Page Header */}
      <PageHeader title="Plan Logs" />
      {/* Contents */}
      <div className="py-6">
        {/* Controls - Select(metric_id), Select(count)*/}
        <div className="flex flex-row space-x-4 px-6">
          {/* Date Range */}
          <div className="flex items-center">
            <div className="form-control mr-2">
              <label className="input-group-sm">
                <input
                  type="date"
                  className="input-bordered input input-sm max-w-[130px] cursor-text px-2 text-center focus:outline-none"
                  max={formatDate(toDayjs)}
                  value={from}
                  onChange={(event) => handleDate('from', event.target.value)}
                />
              </label>
            </div>
            <span>-</span>
            <div className="form-control ml-2">
              <label className="input-group-sm">
                <input
                  type="date"
                  className="input-bordered input input-sm max-w-[130px] cursor-text px-2 text-center focus:outline-none"
                  min={formatDate(fromDayjs)}
                  value={to}
                  onChange={(event) => handleDate('to', event.target.value)}
                />
              </label>
            </div>
          </div>
          {/* Refresh Button */}
          <button
            className="btn-primary btn-sm btn w-32"
            onClick={fetchAutoscalingHistory}
            disabled={isFetching}
          >
            {isFetching ? 'Fetching...' : 'Refresh'}
          </button>
        </div>
        {/* Heatmap */}
        <div className="p-6">
          <div className="wa-card">
            <AutoscalingHistoryHeatmap
              autoscalingHistory={autoscalingHistory}
              from={fromDayjs}
              to={toDayjs}
            />
          </div>
        </div>
        {/* Table */}
        <div className="p-6">
          <div className="wa-card">
            <WASimpleTable<AutoscalingHistoryDefinitionEx>
              tableOptions={{
                data: autoscalingHistory,
                columns,
              }}
            />
          </div>
        </div>
      </div>
    </main>
  );
}
