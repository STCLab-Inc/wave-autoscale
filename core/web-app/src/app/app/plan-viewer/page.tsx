'use client';

import React from 'react';
import PageHeader from '../common/page-header';
import WAVirtualizedTable from '../common/wa-virtualized-table';
import { createColumnHelper } from '@tanstack/react-table';
import dayjs from 'dayjs';
import { useRouter } from 'next/navigation';
import { useScalingPlansWithStats } from '@/services/scaling-plan';
import { ScalingPlanWithStats } from '@/types/scaling-plan-definition-ex';
import EnabledBadge from '../common/enabled-badge';
import { PlanLogCountPerDay } from '@/types/plan-log-stats';
import WATinyBarChart from '../common/wa-tiny-bar-chart';

// Default Values
const DEFAULT_FROM = dayjs().subtract(7, 'days');
const DEFAULT_TO = dayjs();

// Table columns
const columnHelper = createColumnHelper<ScalingPlanWithStats>();
const columns = [
  columnHelper.display({
    id: 'index',
    header: () => '',
    size: 50,
    cell: (cell) => cell.row.index + 1,
  }),
  columnHelper.accessor('id', {
    id: 'id',
    header: () => 'Plan ID',
    cell: (cell: any) => {
      const name = cell.getValue();
      return <div className="font-bold">{name}</div>;
    },
  }),
  columnHelper.accessor('countPerDay', {
    id: 'last_7_days',
    header: () => 'Last 7 Days',
    cell: (cell: any) => {
      const countPerDay = cell.getValue();
      if (!countPerDay) {
        return;
      }
      return (
        <div className="h-14 w-24">
          <WATinyBarChart data={countPerDay} yDataKey="count" xDataKey="date" />
        </div>
      );
    },
  }),
  columnHelper.accessor('countPerDay', {
    id: 'countPerDay',
    header: () => 'Plan Logs',
    cell: (cell: any) => {
      const countPerDay = cell.getValue();
      if (!countPerDay) {
        return;
      }
      const count = countPerDay.reduce(
        (acc: number, curr: PlanLogCountPerDay) => {
          return acc + curr.count;
        },
        0
      );
      return count;
    },
  }),
  columnHelper.accessor('plans', {
    id: 'plans',
    header: () => 'Plan Items',
    cell: (cell: any) => {
      const plans = cell.getValue();
      return plans.length ?? 0;
    },
  }),
  columnHelper.accessor('variables', {
    id: 'variables',
    header: () => 'Variables',
    cell: (cell: any) => {
      const variables = cell.getValue();
      return variables.length ?? 0;
    },
  }),
  columnHelper.accessor('enabled', {
    id: 'enabled',
    header: () => 'Enabled',
    cell: (cell: any) => {
      const enabled = cell.getValue();
      return <EnabledBadge enabled={enabled} />;
    },
  }),
];

export default function PlanViewerPage() {
  const router = useRouter();
  const { data, isLoading } = useScalingPlansWithStats(
    DEFAULT_FROM,
    DEFAULT_TO
  );

  return (
    <main className="flex h-full w-full flex-col">
      {/* Page Header */}
      <PageHeader title="Plan Viewer" />
      <div className="flex flex-1 flex-col py-6">
        {/* Controls */}
        <div className="flex flex-row-reverse px-6">
          <button
            className="btn-primary btn btn-sm font-bold"
            onClick={() => {
              router.push('/app/metrics');
            }}
          >
            Edit Scaling Plan
          </button>
        </div>
        {/* Table */}
        <div className="flex flex-1 flex-col p-6">
          <div className="wa-card flex-1">
            <WAVirtualizedTable<ScalingPlanWithStats>
              tableOptions={{
                data: data ?? [],
                columns,
              }}
              isLoading={isLoading}
              onRowClick={(row) => {
                if (!row.id) {
                  return;
                }
                router.push(`/app/plan-viewer/${row.id}`);
              }}
              rowHeight={80}
            />
          </div>
        </div>
      </div>
    </main>
  );
}
