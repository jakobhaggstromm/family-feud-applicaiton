export default function PlayerList({ players, winnerId, onRemoveTeam }) {
  if (!players || players.length === 0) {
    return (
      <div className="player-list">
        <h3>Teams</h3>
        <p className="no-players">No teams have joined yet.</p>
      </div>
    );
  }

  return (
    <div className="player-list">
      <h3>Teams ({players.length})</h3>
      <ul>
        {players.map((player) => (
          <li key={player.id} className={`player-item ${player.id === winnerId ? 'winner' : ''}`}>
            <span className="player-id">#{player.id}</span>
            <span className="player-name">{player.name}</span>
            <span className={`team-status ${player.is_active ? 'active' : 'inactive'}`}>
              {player.is_active ? 'Active' : 'Offline'}
            </span>
            <span className="team-score">{player.score} pts</span>
            {player.id === winnerId && <span className="winner-badge">⭐ WINNER</span>}
            {onRemoveTeam && (
              <button className="btn btn-small btn-danger" onClick={() => onRemoveTeam(player.id)}>
                Remove
              </button>
            )}
          </li>
        ))}
      </ul>
    </div>
  );
}
