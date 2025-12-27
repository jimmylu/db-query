import { BrowserRouter, Routes, Route, Outlet } from 'react-router-dom';
import { Refine } from '@refinedev/core';
import { RefineKbar, RefineKbarProvider } from '@refinedev/kbar';
import {
  ErrorComponent,
  ThemedLayoutV2,
  ThemedSiderV2,
  ThemedTitleV2,
  useNotificationProvider,
} from '@refinedev/antd';
import { App as AntdApp } from 'antd';
import '@refinedev/antd/dist/reset.css';

import { dataProvider } from './providers/dataProvider';
import { Dashboard } from './pages/Dashboard';
import { QueryPage } from './pages/QueryPage';
import { CrossDatabaseQueryPage } from './pages/CrossDatabaseQueryPage';

function App() {
  return (
    <BrowserRouter>
      <RefineKbarProvider>
        <AntdApp>
          <Refine
            dataProvider={dataProvider}
            notificationProvider={useNotificationProvider}
            resources={[
              {
                name: 'connections',
                list: '/',
              },
              {
                name: 'queries',
                list: '/queries',
              },
              {
                name: 'cross-database',
                list: '/cross-database',
              },
            ]}
            options={{
              syncWithLocation: true,
              warnWhenUnsavedChanges: true,
            }}
          >
            <Routes>
              <Route
                element={
                  <ThemedLayoutV2
                    Sider={() => <ThemedSiderV2 fixed />}
                    Title={({ collapsed }) => (
                      <ThemedTitleV2 collapsed={collapsed} text="DB Query Tool" />
                    )}
                  >
                    <Outlet />
                  </ThemedLayoutV2>
                }
              >
                <Route index element={<Dashboard />} />
                <Route path="queries" element={<QueryPage />} />
                <Route path="cross-database" element={<CrossDatabaseQueryPage />} />
                <Route path="*" element={<ErrorComponent />} />
              </Route>
            </Routes>
            <RefineKbar />
          </Refine>
        </AntdApp>
      </RefineKbarProvider>
    </BrowserRouter>
  );
}

export default App;

