'use client';

import dayjs, { Dayjs } from 'dayjs';
import Link from 'next/link';
import { usePathname, useSearchParams, useRouter } from 'next/navigation';
import { useEffect, useMemo, useState } from 'react';

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
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  // Query Params
  const fromParam = searchParams.get('from');
  const toParam = searchParams.get('to');
  const from = fromParam || formatDate(DEFAULT_FROM);
  const to = toParam || formatDate(DEFAULT_TO);
  const fromDayjs = useMemo(() => dayjs(from).startOf('day'), [from]);
  const toDayjs = useMemo(() => dayjs(to).endOf('day'), [to]);

  // States
  const [topHistoryPlans, setTopHistoryPlans] = useState([
    {
      title: 'EC2 Autoscaling Plan',
    },
    {
      title: 'EC2 Autoscaling Plan',
    },
    {
      title: 'EC2 Autoscaling Plan',
    },
  ]);

  // Effects
  // Fetch Autoscaling History Data when the page is loaded with from and to params
  useEffect(() => {
    // fetchAutoscalingHistory();/
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [fromDayjs, toDayjs]);

  // Handlers
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
    <div className="min-h-full bg-gray-100 p-10">
      <div className="controls mb-10 flex justify-end">
        {/* from - to */}
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
      </div>
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
            <div className="stat-title">Autoscaling Triggered</div>
            <div className="stat-value">50</div>
            {/* <div className="stat-desc">21% more than last month</div> */}
          </div>
        </div>
        {/* History */}
        <div className="stats w-96 shadow">
          <div className="stat">
            <div className="stat-title mb-4">Top Plans</div>
            {topHistoryPlans.map((plan, index) => (
              <div key={index} className="mb-2 flex items-center">
                <div className="badge-primary badge badge-xs mr-4" />
                <div className="text-sm">{plan.title}</div>
              </div>
            ))}
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
            <div className="stat-value">50</div>
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
            <div className="stat-value">50</div>
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
            <div className="stat-value">50</div>
            {/* <div className="stat-desc">21% more than last month</div> */}
          </div>
        </div>
      </div>
      {/* Recent Inflow */}
      <div className="flex">
        <div className="stats flex-1 shadow">
          <div className="stat">
            <div className="stat-title mb-4">Recent Metrics Inflow</div>
            {topHistoryPlans.map((plan, index) => (
              <div key={index} className="mb-2 flex items-center">
                <div className="badge-primary badge badge-xs mr-4" />
                <div className="text-sm">{plan.title}</div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
