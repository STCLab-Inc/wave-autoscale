'use client';

import { useMetricsDataStats } from '@/services/metrics-data';
import PageHeader from './common/page-header';
import PlanLogHeatmap from './plan-log-heatmap';
import { usePlanLogs } from '@/services/plan-log';
import dayjs from 'dayjs';
import WATinyAreaChart from './common/wa-tiny-area-chart';
import { useScalingPlansWithStats } from '@/services/scaling-plan';
import WATinyBarChart from './common/wa-tiny-bar-chart';
import classNames from 'classnames';
import { useScalingComponents } from '@/services/scaling-component';
import ScalingComponentsGrid from './scaling-components-grid';

const PLAN_LOGS_FROM = dayjs().subtract(6, 'days').startOf('day');
const PLAN_LOGS_TO = dayjs().endOf('day');

function MetricsDataStatsItem({
  metricsDataStats,
  isLoading,
}: {
  metricsDataStats: any;
  isLoading: boolean;
}) {
  return (
    <div className="stats">
      <div className="stat">
        <div className={classNames('stat-title mb-4', { skeleton: isLoading })}>
          {metricsDataStats.metricId}
        </div>
        <div
          className={classNames('stat-value flex h-[100px] overflow-hidden', {
            skeleton: isLoading,
          })}
        >
          <div className="flex-1">
            <WATinyAreaChart
              data={metricsDataStats.countPerMinute}
              yDataKey="count"
            />
          </div>
        </div>
      </div>
    </div>
  );
}

function PlanLogsItem({
  planLogsWithStats,
  isLoading,
}: {
  planLogsWithStats: any;
  isLoading: boolean;
}) {
  return (
    <div className="stats">
      <div className="stat">
        <div className={classNames('stat-title mb-4', { skeleton: isLoading })}>
          {planLogsWithStats.id}
        </div>
        <div
          className={classNames('stat-value flex h-[100px] overflow-hidden', {
            skeleton: isLoading,
          })}
        >
          <div className="flex-1">
            <WATinyBarChart
              data={planLogsWithStats.countPerDay}
              yDataKey="count"
              xDataKey="date"
            />
          </div>
        </div>
      </div>
    </div>
  );
}

export default function DashboardPage() {
  const { data: planLogs, isLoading: isPlanLogsLoading } = usePlanLogs(
    undefined,
    PLAN_LOGS_FROM,
    PLAN_LOGS_TO
  );
  const { data: planLogsWithStats, isLoading: isPlanLogsWithStatsLoading } =
    useScalingPlansWithStats(PLAN_LOGS_FROM, PLAN_LOGS_TO);
  const { data: metricsData, isLoading: isMetricsDataLoading } =
    useMetricsDataStats();
  const { data: scalingComponents, isLoading: isScalingComponentsLoading } =
    useScalingComponents();

  return (
    <div className="min-h-full">
      {/* Page Header */}
      <PageHeader title="Dashboard" />
      {/* Contents */}
      <div className="px-6 py-6">
        {/* First Row - Quick Start */}
        <div className="mb-10 flex space-x-4">
          {/* Quick Start */}
          <div className="stats flex-1 !border-none">
            <div className="stat flex h-[200px] bg-gradient-to-r from-[#E9ECFD] to-[#D8C6EE] !py-10">
              {/* Left column */}
              <div className="flex flex-2 flex-col justify-between py-2 pl-10">
                <div className="w-full text-2xl font-bold text-black">
                  Unified Traffic & Autoscaling Management for Clouds and
                  Kubernetes
                </div>
                <div className="flex space-x-2">
                  {/* Buttons */}
                  <a href="/app/templates" className="btn-primary btn btn-sm">
                    See Templates
                  </a>
                  <a
                    href="https://waveautoscale.com/docs"
                    className="btn btn-sm"
                  >
                    Docs
                  </a>
                </div>
              </div>
              <div className="flex flex-1 items-center justify-center">
                <img
                  src="/assets/dashboard/introduction-image.svg"
                  className="max-h-full"
                />
              </div>
            </div>
          </div>
        </div>
        {/* Second Row - Heatmap */}
        <div className="mb-10 flex space-x-4">
          <div className="stats flex-1">
            <div className="stat">
              <div className="stat-title mb-4">Plan Logs</div>
              <div className="stat-value flex h-[200px] overflow-hidden">
                <div
                  className={classNames('flex-1', {
                    skeleton: isPlanLogsLoading,
                  })}
                >
                  <PlanLogHeatmap
                    planLogs={planLogs ?? []}
                    from={PLAN_LOGS_FROM}
                    to={PLAN_LOGS_TO}
                  />
                </div>
                <div className="flex w-48 flex-col justify-start px-4">
                  {isPlanLogsLoading ? (
                    <>
                      <div className="stat-desc skeleton">&nbsp;</div>
                    </>
                  ) : (
                    <>
                      <div className="stat-desc">
                        {`${PLAN_LOGS_FROM.format(
                          'DD MMM'
                        )} ~ ${PLAN_LOGS_TO.format('DD MMM')}`}
                      </div>

                      <div>
                        <div className="stat-title">Total </div>
                        <div className="stat-value">{planLogs?.length}</div>
                      </div>
                    </>
                  )}
                </div>
              </div>
            </div>
          </div>
        </div>
        {/* Third row - Metrics */}
        <div className="mb-10 flex flex-col">
          <div className="mb-4 text-lg font-semibold text-gray-500">
            {`Metrics${metricsData?.length ? ` (${metricsData.length})` : ''}`}
          </div>
          <div className="grid gap-4 md:grid-cols-3 lg:grid-cols-4 2xl:grid-cols-7">
            {metricsData?.map((metricsDataStats) => (
              <MetricsDataStatsItem
                key={metricsDataStats.metricId}
                metricsDataStats={metricsDataStats}
                isLoading={isMetricsDataLoading}
              />
            ))}
            {isMetricsDataLoading && (
              <>
                {
                  // Skeleton
                  Array.from({ length: 3 }).map((_, index) => (
                    <MetricsDataStatsItem
                      key={index}
                      isLoading={true}
                      metricsDataStats={{}}
                    />
                  ))
                }
              </>
            )}
            <div className="flex items-center justify-start pl-10">
              {/* Go to Metrics Viewer */}
              <a href="/app/metrics-viewer" className="btn btn-sm">
                Go to Metrics Viewer
              </a>
            </div>
          </div>
        </div>
        {/* Forth row - Scaling Plans */}
        <div className="mb-10 flex flex-col">
          <div className="mb-4 text-lg font-semibold text-gray-500">
            {`Scaling Plans${
              planLogsWithStats?.length ? ` (${planLogsWithStats.length})` : ''
            }`}
          </div>
          <div className="grid gap-4 md:grid-cols-3 lg:grid-cols-4 2xl:grid-cols-7">
            {planLogsWithStats?.map((planLogWithStats) => (
              <PlanLogsItem
                key={planLogWithStats.id}
                planLogsWithStats={planLogWithStats}
                isLoading={isPlanLogsWithStatsLoading}
              />
            ))}
            {isPlanLogsWithStatsLoading && (
              <>
                {
                  // Skeleton
                  Array.from({ length: 3 }).map((_, index) => (
                    <PlanLogsItem
                      key={index}
                      isLoading={true}
                      planLogsWithStats={{}}
                    />
                  ))
                }
              </>
            )}
            <div className="flex items-center justify-start pl-10">
              {/* Go to Plan Viewer */}
              <a href="/app/plan-viewer" className="btn btn-sm">
                Go to Plan Viewer
              </a>
            </div>
          </div>
        </div>
        {/* Fifth Row - Scaling Components */}
        <div className="mb-10 flex flex-col">
          <div className="mb-4 text-lg font-semibold text-gray-500">
            {`Scaling Components${
              scalingComponents?.length ? ` (${scalingComponents.length})` : ''
            }`}
          </div>
          <ScalingComponentsGrid
            scalingComponents={scalingComponents ?? []}
            isLoading={isScalingComponentsLoading}
          />
        </div>
      </div>
    </div>
  );
}
