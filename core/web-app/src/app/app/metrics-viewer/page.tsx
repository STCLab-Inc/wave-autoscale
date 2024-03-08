'use client';

import React from 'react';
import PageHeader from '../common/page-header';
import { useMetricsDataStats } from '@/services/metrics-data';
import WAVirtualizedTable from '../common/wa-virtualized-table';
import { createColumnHelper } from '@tanstack/react-table';
import { parseDateToDayjs } from '@/utils/date';
import { MetricsDataStats } from '@/types/metrics-data-stats';
import { useRouter } from 'next/navigation';
import WATinyAreaChart from '../common/wa-tiny-area-chart';

// Table columns
const columnHelper = createColumnHelper<MetricsDataStats>();
const columns = [
  columnHelper.display({
    id: 'index',
    header: () => '',
    size: 50,
    cell: (cell) => cell.row.index + 1,
  }),
  columnHelper.accessor('metricId', {
    header: () => 'Metric ID',
    cell: (cell: any) => {
      const name = cell.getValue();
      return (
        <span className="flex items-center justify-start font-bold">
          {name}
        </span>
      );
    },
  }),
  columnHelper.accessor('countPerMinute', {
    header: () => 'Count Per Minute',
    cell: (cell: any) => {
      const countPerMinute: MetricsDataStats['countPerMinute'] =
        cell.getValue();
      if (!countPerMinute) {
        return;
      }
      return (
        <div className="h-14 w-28">
          <WATinyAreaChart data={countPerMinute} yDataKey="count" />
        </div>
      );
    },
  }),
  columnHelper.accessor('timestamps', {
    header: () => 'Last Timestamp',
    cell: (cell: any) => {
      const timestamps = cell.getValue();
      if (!timestamps) {
        return;
      }
      const lastTimestamp = timestamps[timestamps.length - 1];
      return parseDateToDayjs(lastTimestamp)?.format('YYYY-MM-DD HH:mm:ss');
    },
  }),
  columnHelper.accessor('numberOfValues', {
    header: () => 'The Number of Values',
    cell: (cell: any) => cell.getValue(),
  }),
  columnHelper.accessor('lastValues', {
    header: () => 'Last Values',
    cell: (cell: any) => (
      <div className="line-clamp-1 max-w-xl">{cell.getValue()}</div>
    ),
  }),
];

export default function MetricsViewerPage() {
  const router = useRouter();
  const { data, isLoading } = useMetricsDataStats();

  return (
    <main className="flex h-full w-full flex-col">
      {/* Page Header */}
      <PageHeader title="Metrics Viewer" />
      <div className="flex flex-1 flex-col py-6">
        {/* Controls */}
        <div className="flex flex-row-reverse px-6">
          <button
            className="btn-primary btn btn-sm font-bold"
            onClick={() => {
              router.push('/app/metrics');
            }}
          >
            Edit Metric
          </button>
        </div>
        {/* Table */}
        <div className="flex flex-1 flex-col p-6">
          <div className="wa-card flex-1">
            <WAVirtualizedTable<MetricsDataStats>
              tableOptions={{
                data: data ?? [],
                columns,
              }}
              isLoading={isLoading}
              onRowClick={(row) => {
                if (!row.metricId) {
                  return;
                }
                router.push(`/app/metrics-viewer/${row.metricId}`);
              }}
              rowHeight={80}
            />
          </div>
        </div>
      </div>
    </main>
  );
}
