'use client';

import DashboardService, { DashboardStats } from '@/services/dashboard';
import Link from 'next/link';
import { useEffect, useMemo, useState } from 'react';
import PageHeader from './common/page-header';
import { ResponsiveBarCanvas } from '@nivo/bar';
import SCALING_COMPONENT_TEMPLATES from '@/data/wa-templates/scaling-components';
import { TemplateItem } from './common/template-item';
import classNames from 'classnames';

const QUICK_START_ITEMS = [
  // K8s deployment
  SCALING_COMPONENT_TEMPLATES[0].templates[0],
  // ECS Autoscaling
  SCALING_COMPONENT_TEMPLATES[1].templates[1],
  // EC2 Autoscaling
  SCALING_COMPONENT_TEMPLATES[1].templates[0],
];

const LinkIcon = ({ fill }: { fill: string }) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 24 24"
    width="24px"
    height="24px"
    fill={fill}
  >
    <path d="M 19.980469 2.9902344 A 1.0001 1.0001 0 0 0 19.869141 3 L 15 3 A 1.0001 1.0001 0 1 0 15 5 L 17.585938 5 L 8.2929688 14.292969 A 1.0001 1.0001 0 1 0 9.7070312 15.707031 L 19 6.4140625 L 19 9 A 1.0001 1.0001 0 1 0 21 9 L 21 4.1269531 A 1.0001 1.0001 0 0 0 19.980469 2.9902344 z M 5 3 C 3.9069372 3 3 3.9069372 3 5 L 3 19 C 3 20.093063 3.9069372 21 5 21 L 19 21 C 20.093063 21 21 20.093063 21 19 L 21 13 A 1.0001 1.0001 0 1 0 19 13 L 19 19 L 5 19 L 5 5 L 11 5 A 1.0001 1.0001 0 1 0 11 3 L 5 3 z" />
  </svg>
);

export default function DashboardPage() {
  const [stats, setStats] = useState<DashboardStats>();
  const graphData = useMemo(() => {
    if (!stats) {
      return [];
    }

    return [
      {
        id: 'Triggered Plans',
        data: stats.dailyAutoscalingHistory,
      },
    ];
  }, [stats]);
  // Effects
  useEffect(() => {
    const fetch = async () => {
      const stats = await DashboardService.getDashboardStats();
      setStats(stats);
    };
    fetch();
  }, []);

  return (
    <div className="min-h-full">
      {/* Page Header */}
      <PageHeader title="Dashboard" />
      {/* Contents */}
      <div className="px-6 py-6">
        {/* Quick Start */}
        <div className="mb-10 flex space-x-4">
          {/* Graph */}
          <div className="stats flex-1">
            <div className="stat">
              <div className="stat-title">Quick Start</div>
              <div className="stat-value grid grid-cols-4 gap-4 divide-x pt-2">
                {QUICK_START_ITEMS.map((template, index) => (
                  <div
                    key={template.title}
                    className={classNames({ 'pl-4': index !== 0 })}
                  >
                    <TemplateItem
                      key={template.title}
                      template={template}
                      isCard={false}
                    />
                  </div>
                ))}
                {/* See more templates */}
                <div className="flex items-center justify-center p-6 text-base text-wa-gray-700">
                  <Link
                    href="/app/templates"
                    className="flex items-center justify-center"
                  >
                    <span className="text-wrap mr-2">See More Templates</span>
                    <LinkIcon fill="#656669" />
                  </Link>
                </div>
              </div>
            </div>
          </div>
          {/* History */}
          <div className="stats w-96">
            <div className="stat"></div>
          </div>
        </div>
        {/* Autoscaling History Graph */}
        <div className="mb-10 flex space-x-4">
          {/* Graph */}
          <div className="stats flex-1">
            <div className="stat">
              <div className="stat-title">Triggered Plans</div>
              <div className="stat-value h-[300px]">
                <ResponsiveBarCanvas
                  data={stats?.dailyAutoscalingHistory ?? []}
                  margin={{ top: 50, right: 60, bottom: 50, left: 60 }}
                  pixelRatio={2}
                  enableLabel={true}
                  indexBy="date" // minValue="auto"
                  keys={stats?.dailyAutoscalingHistoryPlanIds ?? []}
                  // maxValue="auto"
                  axisBottom={{
                    tickSize: 5,
                    tickPadding: 5,
                    tickRotation: 0,
                    truncateTickAt: 0,
                  }}
                  tooltip={({ indexValue, value, id }) => (
                    <div className="rounded-md border border-wa-gray-600 bg-wa-gray-50 p-2 text-sm font-normal">
                      <div className="">{indexValue}</div>
                      <div className="">{id}</div>
                      <div className="">{value} times</div>
                    </div>
                  )}
                  axisLeft={{
                    tickValues: 5,
                  }}
                  colors={{ scheme: 'green_blue' }}
                  gridYValues={5}
                />
              </div>
            </div>
          </div>
          {/* History */}
          <div className="stats w-96">
            <div className="stat flex flex-col justify-start">
              <div className="stat-title mb-4">Most Triggered Plan IDs</div>
              {stats?.autoscalingHistoryMostTriggered.map((item, index) => (
                <div key={index} className="mb-2 flex items-center">
                  <div className="badge-primary badge badge-xs mr-4" />
                  <div className="text-sm font-semibold">
                    {item[0]} - {item[1]}
                  </div>
                </div>
              ))}
              <div className="stat-desc">
                last {stats?.dailyAutoscalingHistoryKeys?.length} days
              </div>
            </div>
          </div>
        </div>
        {/* Autoscaling History Stats */}
        {/* <div className="mb-10 flex space-x-4">
        <div className="stats flex-1">
          <div className="stat">
            <div className="stat-figure">
              <Link href="/app/autoscaling-history">
                <LinkIcon fill="" />
              </Link>
            </div>
            <div className="stat-title">Triggered Plans</div>
            <div className="stat-value">{stats?.autoscalingHistoryCount}</div>
            <div className="stat-desc">last 7 days</div>
          </div>
        </div>
        <div className="stats w-96">
          <div className="stat">
            <div className="stat-title mb-4">Most Triggered Plan IDs</div>
            {stats?.autoscalingHistoryMostTriggered.map((item, index) => (
              <div key={index} className="mb-2 flex items-center">
                <div className="badge-primary badge badge-xs mr-4" />
                <div className="text-sm">{item[0]}</div>
              </div>
            ))}
            <div className="stat-desc">last 7 days</div>
          </div>
        </div>
      </div> */}
        {/* Recent Inflow */}
        <div className="flex">
          <div className="stats flex-1">
            <div className="stat">
              <div className="stat-title mb-4">Recent Metrics Inflow</div>
              {/* {topHistoryPlans.map((plan, index) => (
              <div key={index} className="mb-2 flex items-center">
                <div className="badge-primary badge badge-xs mr-4" />
                <div className="text-sm">{plan.title}</div>
              </div>
            ))} */}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
