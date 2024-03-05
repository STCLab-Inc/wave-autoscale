'use client';

import React from 'react';
import PageHeader from '../common/page-header';
import WAVirtualizedTable from '../common/wa-virtualized-table';
import { createColumnHelper } from '@tanstack/react-table';
import dayjs from 'dayjs';
import { useRouter } from 'next/navigation';
import {
  useScalingPlans,
  useScalingPlansWithStats,
} from '@/services/scaling-plan';
import {
  ScalingPlanDefinitionEx,
  ScalingPlanWithStats,
} from '@/types/scaling-plan-definition-ex';
import EnabledBadge from '../common/enabled-badge';
import { usePlanLogStats } from '@/services/plan-log';
import {
  PlanLogStats,
  DailyEvent,
} from '@/types/plan-log-stats';
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
    header: () => 'Plan ID',
    cell: (cell: any) => {
      const name = cell.getValue();
      return <div className="font-bold">{name}</div>;
    },
  }),
  columnHelper.accessor('dailyStats', {
    header: () => 'Last 7 Days',
    cell: (cell: any) => {
      const dailyStats = cell.getValue();
      if (!dailyStats) {
        return;
      }
      return (
        <div className="h-14 w-24">
          <WATinyBarChart data={dailyStats} yDataKey="count" xDataKey="date" />
        </div>
      );
    },
  }),
  columnHelper.accessor('dailyStats', {
    header: () => 'Triggered Events',
    cell: (cell: any) => {
      const dailyStats = cell.getValue();
      if (!dailyStats) {
        return;
      }
      const count = dailyStats.reduce((acc: number, curr: DailyEvent) => {
        return acc + curr.count;
      }, 0);
      return count;
    },
  }),
  columnHelper.accessor('plans', {
    header: () => 'Plan Items',
    cell: (cell: any) => {
      const plans = cell.getValue();
      return plans.length ?? 0;
    },
  }),
  columnHelper.accessor('variables', {
    header: () => 'Variables',
    cell: (cell: any) => {
      const variables = cell.getValue();
      return variables.length ?? 0;
    },
  }),
  columnHelper.accessor('enabled', {
    header: () => 'Enabled',
    cell: (cell: any) => {
      const enabled = cell.getValue();
      return <EnabledBadge enabled={enabled} />;
    },
  }),
];

export default function PlanViewerPage() {
  const router = useRouter();
  // const [data, setData] = useState<MetricsDataStats[]>([]);
  const { data, isLoading } = useScalingPlansWithStats(
    DEFAULT_FROM,
    DEFAULT_TO
  );

  console.log({ data });
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
