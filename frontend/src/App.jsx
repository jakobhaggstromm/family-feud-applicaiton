import { useState, useEffect, useCallback, useRef } from 'react';
import { useWebSocket } from './hooks/useWebSocket';
import JoinScreen from './components/JoinScreen';
import BuzzerScreen from './components/BuzzerScreen';
import HostPanel from './components/HostPanel';
import DisplayBoard from './components/DisplayBoard';
import './App.css';

function App() {
  const { gameState, lastMessage, connected, sendMessage } = useWebSocket();
  const pathname = window.location.pathname.toLowerCase();
  const page = pathname.includes('/admin') ? 'admin' : pathname.includes('/display') ? 'display' : 'player';
  const [playerName, setPlayerName] = useState('');
  const [token, setToken] = useState(() => sessionStorage.getItem('buzzer_token'));
  const [adminToken, setAdminToken] = useState(() => sessionStorage.getItem('admin_token'));
  const [adminPassword, setAdminPassword] = useState('');
  const [adminStatus, setAdminStatus] = useState(page === 'admin' ? 'pending' : 'idle');
  const [adminError, setAdminError] = useState('');

  const pendingJoinRef = useRef(false);
  const pendingLeaveRef = useRef(false);

  useEffect(() => {
    document.body.classList.toggle('display-mode', page === 'display');
    return () => {
      document.body.classList.remove('display-mode');
    };
  }, [page]);

  useEffect(() => {
    setAdminStatus(page === 'admin' && adminToken ? 'pending' : 'idle');
  }, [page]);

  useEffect(() => {
    if (!lastMessage) return;

    if (lastMessage.type === 'admin_auth_result') {
      if (lastMessage.success) {
        setAdminStatus('granted');
        setAdminError('');
        setAdminToken(lastMessage.token);
        sessionStorage.setItem('admin_token', lastMessage.token);
      } else {
        if (adminToken) {
          setAdminToken(null);
          sessionStorage.removeItem('admin_token');
        }
        setAdminStatus('denied');
        setAdminError(lastMessage.error || 'Admin authentication failed');
      }
    }

    if (lastMessage.type === 'join_result' && lastMessage.success && pendingJoinRef.current) {
      pendingJoinRef.current = false;
      setToken(lastMessage.token);
      sessionStorage.setItem('buzzer_token', lastMessage.token);
    }

    if (lastMessage.type === 'leave_result' && lastMessage.success && pendingLeaveRef.current) {
      pendingLeaveRef.current = false;
      setToken(null);
      sessionStorage.removeItem('buzzer_token');
    }
  }, [lastMessage]);

  useEffect(() => {
    if (connected && token && page === 'player') {
      pendingJoinRef.current = true;
      sendMessage({ action: 'join', name: playerName || 'Reconnecting', token });
    }
  }, [connected, token, page, playerName, sendMessage]);

  useEffect(() => {
    if (connected && page === 'admin' && adminToken) {
      setAdminStatus('pending');
      setAdminError('');
      sendMessage({ action: 'claim_admin', token: adminToken, password: null });
    }
  }, [connected, page, adminToken, sendMessage]);

  useEffect(() => {
    if (connected) {
      sendMessage({ action: 'get_state' });
    }
  }, [connected, sendMessage]);

  const handleJoin = useCallback((name) => {
    setPlayerName(name);
    pendingJoinRef.current = true;
    sendMessage({ action: 'join', name });
  }, [sendMessage]);

  const handleBuzz = useCallback(() => {
    if (token) {
      sendMessage({ action: 'buzz', token });
    }
  }, [sendMessage, token]);

  const handleLeave = useCallback(() => {
    if (token) {
      pendingLeaveRef.current = true;
      sendMessage({ action: 'leave', token });
    }
  }, [sendMessage, token]);

  const handleHostAction = useCallback((action) => {
    if (adminStatus === 'granted') {
      sendMessage(action);
    }
  }, [adminStatus, sendMessage]);

  const handleAdminLogin = useCallback((event) => {
    event.preventDefault();
    if (!adminPassword.trim()) {
      setAdminError('Enter the admin password');
      return;
    }

    setAdminStatus('pending');
    setAdminError('');
    sendMessage({ action: 'claim_admin', token: null, password: adminPassword });
  }, [adminPassword, sendMessage]);

  return (
    <div className={`app ${page === 'display' ? 'app--display' : ''}`}>
      <div className={`connection-status ${connected ? 'connected' : 'disconnected'}`}>
        {connected ? 'Connected' : 'Reconnecting...'}
      </div>

      <div className="page-nav">
        <a href="/player" className={`nav-link ${page === 'player' ? 'active' : ''}`}>Player</a>
        <a href="/admin" className={`nav-link ${page === 'admin' ? 'active' : ''}`}>Admin</a>
        <a href="/display" className={`nav-link ${page === 'display' ? 'active' : ''}`}>Display</a>
      </div>

      {page === 'player' && !token && <JoinScreen onJoin={handleJoin} />}

      {page === 'player' && token && (
        <BuzzerScreen
          gameState={gameState}
          lastMessage={lastMessage}
          onBuzz={handleBuzz}
          onLeave={handleLeave}
          playerName={playerName}
        />
      )}

      {page === 'admin' && connected && adminStatus === 'granted' && (
        <HostPanel
          gameState={gameState}
          lastMessage={lastMessage}
          onAction={handleHostAction}
        />
      )}

      {page === 'admin' && !connected && (
        <div className="access-panel">
          <h2>Connecting Admin Panel</h2>
          <p>Waiting for the server connection...</p>
        </div>
      )}

      {page === 'admin' && connected && adminStatus === 'pending' && (
        <div className="access-panel">
          <h2>Connecting Admin Panel</h2>
          <p>Authenticating admin access...</p>
        </div>
      )}

      {page === 'admin' && connected && (adminStatus === 'idle' || adminStatus === 'denied') && (
        <div className="access-panel locked">
          <h2>Admin Login</h2>
          <p>Enter the admin password to claim the control panel.</p>
          <form className="join-form" onSubmit={handleAdminLogin}>
            <input
              type="password"
              placeholder="Admin password"
              value={adminPassword}
              onChange={(event) => setAdminPassword(event.target.value)}
              autoFocus
            />
            <button className="btn btn-primary" type="submit">Claim Admin</button>
          </form>
          {adminError && <p className="admin-error">{adminError}</p>}
        </div>
      )}

      {page === 'display' && <DisplayBoard gameState={gameState} />}
    </div>
  );
}

export default App;
