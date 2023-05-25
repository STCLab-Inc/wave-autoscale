'use client';

import ContentHeader from '../content-header';
import AutoscalingHistoryService from '@/services/autoscaling-history';
import dayjs, { Dayjs } from 'dayjs';
import { AutoscalingHistoryDefinition } from '@/types/bindings/autoscaling-history-definition';
import { renderKeyValuePairsWithJson } from '../keyvalue-renderer';
import HistoryHeatmap from './history-heatmap';
import { useParams, useSearchParams } from 'next/navigation';
import { useRouter } from 'next/navigation';
import { useEffect, useMemo, useState } from 'react';

async function getHistory(
  from: Dayjs = dayjs().subtract(7, 'days'),
  to: Dayjs = dayjs()
) {
  const history = await AutoscalingHistoryService.getHistoryByFromTo(from, to);
  return history;
}

const DEFAULT_FROM = dayjs().subtract(7, 'days');
const DEFAULT_TO = dayjs();

export default function AutoscalingHistoryPage({}) {
  const searchParams = useSearchParams();
  const from = searchParams.get('from');
  const to = searchParams.get('to');
  const fromDayjs = useMemo(() => (from ? dayjs(from) : DEFAULT_FROM), [from]);
  const toDayjs = useMemo(() => (to ? dayjs(to) : DEFAULT_TO), [to]);
  const [history, setHistory] = useState<AutoscalingHistoryDefinition[]>([]);
  const router = useRouter();

  console.log(fromDayjs, toDayjs);
  useEffect(() => {
    const fetchHistory = async () => {
      const history = await getHistory(fromDayjs, toDayjs);
      setHistory(history);
    };
    fetchHistory();
  }, [fromDayjs, toDayjs]);

  return (
    <main className="relative flex flex-1 flex-col">
      <ContentHeader
        title="Autoscaling History"
        right={
          <div className="flex items-center">
            <div className="flex items-center">
              <div className="mr-2">From</div>
              <input
                type="date"
                className="input-bordered input"
                defaultValue={fromDayjs?.format('YYYY-MM-DD')}
                onChange={(e) => {
                  const from = dayjs(e.target.value);
                  router.push(
                    `/app/autoscaling-history?from=${from.format(
                      'YYYY-MM-DD'
                    )}&to=${toDayjs?.format('YYYY-MM-DD')}`
                  );
                }}
              />
            </div>
            <div className="flex items-center">
              <div className="mr-2">To</div>
              <input
                type="date"
                className="input-bordered input"
                defaultValue={toDayjs?.format('YYYY-MM-DD')}
                onChange={(e) => {
                  const to = dayjs(e.target.value);
                  router.push(
                    `/app/autoscaling-history?from=${fromDayjs?.format(
                      'YYYY-MM-DD'
                    )}&to=${to.format('YYYY-MM-DD')}`
                  );
                }}
              />
            </div>
          </div>
        }
      />
      <div className="w-full overflow-x-auto">
        <HistoryHeatmap history={history} from={fromDayjs} to={toDayjs} />
        <table className="table-compact table w-full">
          {/* head */}
          <thead>
            <tr>
              <th>
                <label>
                  <input type="checkbox" className="checkbox" />
                </label>
              </th>
              <th>Plan ID</th>
              <th>Plan Item</th>
              <th>Metric Values</th>
              <th>Metadata Values</th>
              <th>Fail Message</th>
              <th>Status</th>
              <th>Date</th>
            </tr>
          </thead>
          <tbody>
            {history.map((historyItem: AutoscalingHistoryDefinition) => {
              // console.log({
              //   date: historyItem.created_at,
              //   type: typeof historyItem.created_at,
              // });
              return (
                <tr key={historyItem.id}>
                  <th>
                    <label>
                      <input type="checkbox" className="checkbox" />
                    </label>
                  </th>
                  <td>{historyItem.plan_id}</td>
                  <td>
                    {renderKeyValuePairsWithJson(historyItem.plan_item_json)}
                  </td>
                  <td>
                    {renderKeyValuePairsWithJson(
                      historyItem.metric_values_json
                    )}
                  </td>
                  <td>
                    {renderKeyValuePairsWithJson(
                      historyItem.metadata_values_json
                    )}
                  </td>
                  <td>
                    {historyItem.fail_message &&
                      renderKeyValuePairsWithJson(historyItem.fail_message)}
                  </td>
                  <td>
                    {historyItem.fail_message ? (
                      <div className="badge-error badge">Failed</div>
                    ) : (
                      <div className="badge-success badge">Success</div>
                    )}
                  </td>
                  <td>
                    {dayjs
                      .unix(historyItem.created_at)
                      .format('YYYY-MM-DD HH:mm:ss')}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </main>
  );
}
