'use client';

import React, { useEffect, useMemo, useState } from 'react';
import dayjs, { Dayjs } from 'dayjs';
import { useSearchParams } from 'next/navigation';
import { useRouter } from 'next/navigation';
import { decodeTime } from 'ulid';

import ContentHeader from '../content-header';
import AutoscalingHistoryService from '@/services/autoscaling-history';
import { AutoscalingHistoryDefinition } from '@/types/bindings/autoscaling-history-definition';
import HistoryHeatmap from './history-heatmap';
import { renderKeyValuePairsWithJson } from '../keyvalue-renderer';

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
  const from = searchParams.get('from') || DEFAULT_FROM.format('YYYY-MM-DD');
  const to = searchParams.get('to') || DEFAULT_TO.format('YYYY-MM-DD');
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

  const handleFromDateChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newFromDate = dayjs(e.target.value);
    router.push(
      `/app/autoscaling-history?from=${newFromDate.format(
        'YYYY-MM-DD'
      )}&to=${to}`
    );
  };

  const handleToDateChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newToDate = dayjs(e.target.value);
    router.push(
      `/app/autoscaling-history?from=${from}&to=${newToDate.format(
        'YYYY-MM-DD'
      )}`
    );
  };

  const [checkAllFlag, setCheckAllFlag] = useState(false);

  const handleCheckAllChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const checked = e.target.checked;
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
            <div className="form-control mr-1">
              <label className="input-group-sm">
                <input
                  type="date"
                  className="input-bordered input input-sm max-w-[130px] cursor-text focus:outline-none"
                  max={toDayjs.format('YYYY-MM-DD')}
                  value={from}
                  onChange={handleFromDateChange}
                />
              </label>
            </div>
            <span>-</span>
            <div className="form-control ml-1">
              <label className="input-group-sm">
                <input
                  type="date"
                  className="input-bordered input input-sm max-w-[130px] cursor-text focus:outline-none"
                  min={from}
                  value={to}
                  onChange={handleToDateChange}
                />
              </label>
            </div>
          </div>
        }
      />
      <div className="flex w-full flex-col">
        <HistoryHeatmap history={history} from={fromDayjs} to={toDayjs} />
        <table className="flex w-full flex-col">
          <thead className="text-md flex h-12 w-full items-center justify-between border-b border-t bg-gray-200 px-8 py-0 font-bold text-gray-800">
            <tr className="flex h-full w-full">
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
              <th className="mx-4 flex h-full w-full flex-3 items-center">
                <span className="flex items-center break-words">Plan ID</span>
              </th>
              <th className="mx-4 flex h-full w-full flex-10 items-center">
                <span className="flex items-center break-words">Plan Item</span>
              </th>
              <th className="mx-4 flex h-full w-full flex-8 items-center">
                <span className="flex items-center break-words">
                  Metric Values
                </span>
              </th>
              <th className="mx-4 flex h-full w-full flex-8 items-center">
                <span className="flex items-center break-words">
                  Metadata Values
                </span>
              </th>
              <th className="mx-4 flex h-full w-full flex-8 items-center">
                <span className="flex items-center break-words">
                  Fail Message
                </span>
              </th>
              <th className="mx-4 flex h-full w-full flex-2 items-center">
                <div className="items-centerbreak-words flex">Status</div>
              </th>
              <th className="mx-4 flex h-full w-full flex-5 items-center">
                <div className="items-centerbreak-words flex">Date</div>
              </th>
            </tr>
          </thead>
          <tbody className="text-md min-h-12 flex w-full flex-col items-center justify-between border-b border-t px-8 py-0 text-gray-800">
            {history.map((historyItem: AutoscalingHistoryDefinitionEx) => (
              <tr key={historyItem.id} className="flex h-full w-full py-4">
                <td className="mr-4 flex h-full flex-1 items-start">
                  <label className="flex items-center">
                    <input
                      type="checkbox"
                      className="checkbox"
                      checked={historyItem.isChecked}
                      onChange={(e) => {
                        const checked = e.target.checked;
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
                <td className="mx-4 flex h-full w-full flex-3 items-start">
                  <div className="flex items-center break-all">
                    {historyItem.plan_id}
                  </div>
                </td>
                <td className="mx-4 flex h-full w-full flex-10 items-start">
                  <div className="flex flex-col items-center">
                    {renderKeyValuePairsWithJson(
                      historyItem.plan_item_json,
                      false
                    )}
                  </div>
                </td>
                <td className="mx-4 flex h-full w-full flex-8 items-start">
                  <div className="flex flex-col items-center">
                    {renderKeyValuePairsWithJson(
                      historyItem.metric_values_json,
                      false
                    )}
                  </div>
                </td>
                <td className="mx-4 flex h-full w-full flex-8 items-start">
                  <div className="flex flex-col items-center">
                    {renderKeyValuePairsWithJson(
                      historyItem.metadata_values_json,
                      false
                    )}
                  </div>
                </td>
                <td className="mx-4 flex h-full w-full flex-8 items-start">
                  <div className="flex flex-col items-center">
                    {historyItem.fail_message &&
                      renderKeyValuePairsWithJson(
                        historyItem.fail_message,
                        false
                      )}
                  </div>
                </td>
                <td className="mx-4 flex h-full w-full flex-2 items-start">
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
                <td className="mx-4 flex h-full w-full flex-5 items-start">
                  <div className="flex items-center break-all">
                    {dayjs
                      .unix(historyItem.created_at / 1000)
                      .format('YYYY/MM/DD HH:mm:ss')}
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
