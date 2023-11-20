'use client';

import React, { useEffect, useMemo, useState } from 'react';
import dayjs, { Dayjs } from 'dayjs';
import { useSearchParams } from 'next/navigation';
import { useRouter } from 'next/navigation';
import { decodeTime } from 'ulid';
import Link from 'next/link';

import ContentHeader from '../content-header';
import AutoscalingHistoryService from '@/services/autoscaling-history';
import { AutoscalingHistoryDefinition } from '@/types/bindings/autoscaling-history-definition';
import HistoryHeatmap from './history-heatmap';
import { renderKeyValuePairsWithJson } from '../keyvalue-renderer';

const formatDate = (date: Dayjs) => date.format('YYYY-MM-DD');

async function getHistory(from: Dayjs, to: Dayjs) {
  const history = await AutoscalingHistoryService.getHistoryByFromTo(from, to);
  return history;
}

const DEFAULT_FROM = dayjs().subtract(7, 'days');
const DEFAULT_TO = dayjs();

interface AutoscalingHistoryDefinitionEx extends AutoscalingHistoryDefinition {
  created_at: number;
  isChecked: boolean;
}

export default function AutoscalingHistoryPage() {
  const searchParams = useSearchParams();
  const fromParam = searchParams.get('from');
  const toParam = searchParams.get('to');
  const from = fromParam || formatDate(DEFAULT_FROM);
  const to = toParam || formatDate(DEFAULT_TO);
  const fromDayjs = useMemo(() => dayjs(from), [from]);
  const toDayjs = useMemo(() => dayjs(to).endOf('day'), [to]);
  const [history, setHistory] = useState<AutoscalingHistoryDefinitionEx[]>([]);
  const router = useRouter();

  useEffect(() => {
    const fetchHistory = async () => {
      try {
        let history = await getHistory(fromDayjs, toDayjs);
        history = history.map((historyItem: AutoscalingHistoryDefinition) => ({
          ...historyItem,
          created_at: decodeTime(historyItem.id),
        }));
        setHistory(history);
      } catch (error) {
        console.error({ error });
      }
    };
    fetchHistory();
  }, [fromDayjs, toDayjs]);

  const handleDateChange = (field: 'from' | 'to', value: string) => {
    const newDate = dayjs(value);
    const isValidDate = newDate.isValid();

    if (isValidDate) {
      const params = {
        from: field === 'from' ? formatDate(newDate) : from,
        to: field === 'to' ? formatDate(newDate) : to,
      };
      router.push(
        `/app/autoscaling-history?from=${params.from}&to=${params.to}`
      );
    }
  };
  const [checkAllFlag, setCheckAllFlag] = useState(false);

  const handleCheckAllChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const checked = event.target.checked;
    setCheckAllFlag(checked);
    const updatedHistory = history.map((historyItem) => ({
      ...historyItem,
      isChecked: checked,
    }));
    setHistory(updatedHistory);
  };

  return (
    <main className="flex h-full w-full flex-col">
      <ContentHeader
        title="Autoscaling History"
        right={
          <div className="flex items-center">
            <div className="form-control mr-2">
              <label className="input-group-sm">
                <input
                  type="date"
                  className="input-bordered input input-sm max-w-[130px] cursor-text px-2 text-center focus:outline-none"
                  max={formatDate(toDayjs)}
                  value={from}
                  onChange={(event) =>
                    handleDateChange('from', event.target.value)
                  }
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
                  onChange={(event) =>
                    handleDateChange('to', event.target.value)
                  }
                />
              </label>
            </div>
          </div>
        }
      />
      <div className="flex w-full flex-col">
        <HistoryHeatmap history={history} from={fromDayjs} to={toDayjs} />
        <table className="flex w-full flex-col">
          <thead className="text-md flex h-12 w-full items-center justify-between border-b border-t bg-gray-75 py-0 font-bold text-gray-800">
            <tr className="flex h-full w-full px-8">
              <th className="mr-4 flex h-full flex-1 items-center">
                <label className="flex h-full items-center">
                  <input
                    type="checkbox"
                    className="checkbox"
                    checked={checkAllFlag}
                    onChange={handleCheckAllChange}
                  />
                </label>
              </th>
              <th className="mx-4 flex h-full w-full flex-4 items-center">
                <span className="flex items-center break-words">Plan ID</span>
              </th>
              <th className="mx-4 flex h-full w-full flex-10 items-center">
                <span className="flex items-center break-words">
                  Metric Values
                </span>
              </th>
              <th className="mx-4 flex h-full w-full flex-5 items-center">
                <span className="flex items-center break-words">
                  Metadata Values
                </span>
              </th>
              <th className="mx-4 flex h-full w-full flex-3 items-center">
                <span className="flex items-center break-words">
                  Fail Message
                </span>
              </th>
              <th className="mx-4 flex h-full w-full flex-1 items-center">
                <div className="flex items-center break-words">Status</div>
              </th>
              <th className="mx-4 flex h-full w-full flex-2 items-center">
                <div className="flex items-center break-words">Date</div>
              </th>
              <th className="mx-4 flex h-full w-full flex-1 items-center">
                <div className="flex items-center break-words">Actions</div>
              </th>
            </tr>
          </thead>
          <tbody className="text-md min-h-12 flex w-full flex-col items-center justify-between border-b py-0 text-gray-800">
            {history.map((historyItem: AutoscalingHistoryDefinitionEx) => (
              <tr
                key={historyItem.id}
                className="flex w-full border-b px-8 py-4"
              >
                <td className="mr-4 flex h-full flex-1 items-start">
                  <label className="flex items-center">
                    <input
                      type="checkbox"
                      className="checkbox"
                      checked={historyItem.isChecked}
                      onChange={(event) => {
                        const checked = event.target.checked;
                        const updatedHistory = history.map((item) =>
                          item.id === historyItem.id
                            ? { ...item, isChecked: checked }
                            : item
                        );
                        setHistory(updatedHistory);
                      }}
                    />
                  </label>
                </td>
                <td className="mx-4 flex h-full w-full flex-4 items-start">
                  <div className="flex items-center break-all">
                    {historyItem.plan_id}
                  </div>
                </td>
                <td className="mx-4 flex h-full w-full flex-10 items-start">
                  <div className="flex flex-col items-center">
                    {renderKeyValuePairsWithJson(
                      historyItem.metric_values_json,
                      false
                    )}
                  </div>
                </td>
                <td className="mx-4 flex h-full w-full flex-5 items-start">
                  <div className="flex flex-col items-center">
                    {renderKeyValuePairsWithJson(
                      historyItem.metadata_values_json,
                      false
                    )}
                  </div>
                </td>
                <td className="mx-4 flex h-full w-full flex-3 items-start">
                  <div className="flex flex-col items-center">
                    {historyItem.fail_message &&
                      renderKeyValuePairsWithJson(
                        historyItem.fail_message,
                        false
                      )}
                  </div>
                </td>
                <td className="mx-4 flex h-full w-full flex-1 items-start">
                  <div className="flex items-center">
                    {historyItem.fail_message ? (
                      <div className="badge-error badge bg-[#E0242E] px-2 py-3 text-white">
                        Failed
                      </div>
                    ) : (
                      <div className="badge-success badge bg-[#074EAB] px-2 py-3 text-white">
                        Success
                      </div>
                    )}
                  </div>
                </td>
                <td className="mx-4 flex h-full w-full flex-2 items-start">
                  <div className="flex items-center break-all">
                    {dayjs
                      .unix(historyItem.created_at / 1000)
                      .format('YYYY/MM/DD HH:mm:ss')}
                  </div>
                </td>
                <td className="mx-4 flex h-full w-full flex-1 items-start">
                  <div className="flex items-center">
                    <Link
                      href={`/app/autoscaling-history/${historyItem.id}?from=${from}&to=${to}`}
                    >
                      <button className="badge-success badge bg-[#074EAB] px-2 py-3 text-white">
                        Details
                      </button>
                    </Link>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </main>
  );
}
