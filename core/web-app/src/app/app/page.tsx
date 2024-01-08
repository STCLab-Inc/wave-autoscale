'use client';

import DashboardService, { DashboardStats } from '@/services/dashboard';
import dayjs, { Dayjs } from 'dayjs';
import Link from 'next/link';
import { useEffect, useState } from 'react';

// Default Values
const DEFAULT_FROM = dayjs().subtract(7, 'days');
const DEFAULT_TO = dayjs();
const formatDate = (date: Dayjs) => date.format('YYYY-MM-DD');

const LinkIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 24 24"
    width="24px"
    height="24px"
  >
    <path d="M 19.980469 2.9902344 A 1.0001 1.0001 0 0 0 19.869141 3 L 15 3 A 1.0001 1.0001 0 1 0 15 5 L 17.585938 5 L 8.2929688 14.292969 A 1.0001 1.0001 0 1 0 9.7070312 15.707031 L 19 6.4140625 L 19 9 A 1.0001 1.0001 0 1 0 21 9 L 21 4.1269531 A 1.0001 1.0001 0 0 0 19.980469 2.9902344 z M 5 3 C 3.9069372 3 3 3.9069372 3 5 L 3 19 C 3 20.093063 3.9069372 21 5 21 L 19 21 C 20.093063 21 21 20.093063 21 19 L 21 13 A 1.0001 1.0001 0 1 0 19 13 L 19 19 L 5 19 L 5 5 L 11 5 A 1.0001 1.0001 0 1 0 11 3 L 5 3 z" />
  </svg>
);

export default function Dashboard() {
  const [stats, setStats] = useState<DashboardStats>();
  // Effects
  useEffect(() => {
    const fetch = async () => {
      const stats = await DashboardService.getDashboardStats();
      setStats(stats);
    };
    fetch();
  }, []);

  return (
    <div className="min-h-full bg-gray-100 p-10">
      {/* Autoscaling History Stats */}
      <div className="mb-10 flex space-x-4">
        {/* History Count */}
        <div className="stats flex-1 shadow">
          <div className="stat">
            <div className="stat-figure">
              <Link href="/app/autoscaling-history">
                <LinkIcon />
              </Link>
            </div>
            <div className="stat-title">Triggered Plans</div>
            <div className="stat-value">{stats?.autoscalingHistoryCount}</div>
            <div className="stat-desc">last 7 days</div>
          </div>
        </div>
        {/* History */}
        <div className="stats w-96 shadow">
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
      </div>
      {/* Definition Stats */}
      <div className="grid-4 mb-10 grid grid-cols-3 gap-4">
        {/* Metric Definitions */}
        <div className="stats shadow">
          <div className="stat">
            <div className="stat-figure">
              <Link href="/app/metrics">
                <LinkIcon />
              </Link>
            </div>
            <div className="stat-title">Metric Definitions</div>
            <div className="stat-value">{stats?.metricsCount}</div>
            {/* <div className="stat-desc">21% more than last month</div> */}
          </div>
        </div>
        {/* Scaling Component Definitions */}
        <div className="stats shadow">
          <div className="stat">
            <div className="stat-figure">
              <Link href="/app/scaling-components">
                <LinkIcon />
              </Link>
            </div>
            <div className="stat-title">Scaling Component Definitions</div>
            <div className="stat-value">{stats?.scalingComponentsCount}</div>
            {/* <div className="stat-desc">21% more than last month</div> */}
          </div>
        </div>
        {/* Plan Definitions */}
        <div className="stats shadow">
          <div className="stat">
            <div className="stat-figure">
              <Link href="/app/scaling-plans">
                <LinkIcon />
              </Link>
            </div>
            <div className="stat-title">Plan Definitions</div>
            <div className="stat-value">{stats?.plansCount}</div>
            {/* <div className="stat-desc">21% more than last month</div> */}
          </div>
        </div>
      </div>
      {/* Recent Inflow */}
      <div className="flex">
        <div className="stats flex-1 shadow">
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
  );
}
