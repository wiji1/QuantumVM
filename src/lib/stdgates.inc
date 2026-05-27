// Standard gates library
// All gates defined in terms of U and CX primitives

// Single qubit gates
gate x q { U(pi, 0, pi) q; }
gate y q { U(pi, pi/2, pi/2) q; }
gate z q { U(0, 0, pi) q; }
gate h q { U(pi/2, 0, pi) q; }
gate s q { pow(1/2) @ z q; }
gate sdg q { inv @ s q; }
gate t q { pow(1/4) @ z q; }
gate tdg q { inv @ t q; }
gate sx q { pow(1/2) @ x q; }
gate sxdg q { inv @ sx q; }

// Rotation gates
gate rx(theta) q { U(theta, -pi/2, pi/2) q; }
gate ry(theta) q { U(theta, 0, 0) q; }
gate rz(theta) q { inv @ U(0, 0, -theta) q; }

// Phase gate
gate p(lambda) q { U(0, 0, lambda) q; }

// General unitary with euler angles
gate u1(lambda) q { U(0, 0, lambda) q; }
gate u2(phi, lambda) q { U(pi/2, phi, lambda) q; }
gate u3(theta, phi, lambda) q { U(theta, phi, lambda) q; }

// Two qubit gates
gate cx ctrl, tgt { ctrl @ x ctrl, tgt; }
gate cy ctrl, tgt { ctrl @ y ctrl, tgt; }
gate cz ctrl, tgt { ctrl @ z ctrl, tgt; }
gate ch ctrl, tgt { ctrl @ h ctrl, tgt; }
gate swap a, b { cx a, b; cx b, a; cx a, b; }
gate crx(theta) ctrl, tgt { ctrl @ rx(theta) ctrl, tgt; }
gate cry(theta) ctrl, tgt { ctrl @ ry(theta) ctrl, tgt; }
gate crz(theta) ctrl, tgt { ctrl @ rz(theta) ctrl, tgt; }
gate cp(lambda) ctrl, tgt { ctrl @ p(lambda) ctrl, tgt; }

// Three qubit gates
gate ccx a, b, c { ctrl @ ctrl @ x a, b, c; }
gate cswap a, b, c { ctrl @ swap a, b, c; }