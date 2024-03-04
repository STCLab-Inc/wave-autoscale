'use client';

import React from 'react';
import PageHeader from '../common/page-header';
import { useMetricsDataStats } from '@/services/metrics-data';
import WAVirtualizedTable from '../common/wa-virtualized-table';
import { createColumnHelper } from '@tanstack/react-table';
import dayjs from 'dayjs';
import { parseDateToDayjs } from '@/utils/date';
import { MetricsDataStats } from '@/types/metrics-data-stats';
import { useRouter } from 'next/navigation';
import WATinyAreaChart from '../common/wa-tiny-area-chart';

const MINUTES_AGO = dayjs().subtract(5, 'minute');
const NUMBER_OF_SECONDS = dayjs().diff(MINUTES_AGO, 'second');
// Table columns
const columnHelper = createColumnHelper<MetricsDataStats>();
const columns = [
  columnHelper.accessor('metricId', {
    header: () => 'Metric ID',
    cell: (cell: any) => {
      const name = cell.getValue();
      2;
      return (
        <span className="flex items-center justify-start font-bold">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            strokeWidth={1.5}
            stroke="currentColor"
            className="h-4 w-4 mr-2"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              d="M13.5 4.5 21 12m0 0-7.5 7.5M21 12H3"
            />
          </svg>
          {name}
        </span>
      );
    },
  }),
  columnHelper.accessor('timestampFrequency', {
    header: () => 'Frequency',
    cell: (cell: any) => {
      const timestampFrequency = cell.getValue();
      if (!timestampFrequency) {
        return;
      }
      console.log({ timestampFrequency });
      return (
        <div className="h-14 w-24">
          <WATinyAreaChart data={timestampFrequency} dataKey="y" />
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
      return parseDateToDayjs(lastTimestamp).format('YYYY-MM-DD HH:mm:ss');
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
  // const [data, setData] = useState<MetricsDataStats[]>([]);
  const { data, isLoading } = useMetricsDataStats();

  return (
    <main className="flex h-full w-full flex-col">
      {/* Page Header */}
      <PageHeader title="Metrics Viewer" />
      <div className="flex flex-1 flex-col py-6">
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
