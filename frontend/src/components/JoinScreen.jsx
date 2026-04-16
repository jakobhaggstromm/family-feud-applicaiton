import { useState } from 'react';

export default function JoinScreen({ onJoin }) {
  const [name, setName] = useState('');

  const handleSubmit = (e) => {
    e.preventDefault();
    if (name.trim()) {
      onJoin(name.trim());
    }
  };

  return (
    <div className="join-screen">
      <div className="logo">
        <h1>FAMILY FEUD</h1>
        <p className="subtitle">BUZZER</p>
      </div>

      <form onSubmit={handleSubmit} className="join-form">
        <input
          type="text"
          placeholder="Enter your team name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          autoFocus
          maxLength={20}
        />
        <button type="submit" className="btn btn-primary" disabled={!name.trim()}>
          Join Game
        </button>
      </form>
    </div>
  );
}
