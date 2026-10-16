# CPU lock 변경 후 Windows GNU 기본 workspace 검증

2026-10-03 KST. `scripts/windows_native_probe.py`를 수정 없이 사용했다. private CLI desktop의 owned Job 안에서 네 작업을 실행했으며 모든 로그에서 input desktop unchanged가 확인됐다. 프로젝트 Windows 전용 cargo-home/target/tmp를 사용했고 Linux Cargo 환경과 분리했다.

| 실제 명령 | 결과 | 경과 |
| --- | --- | --- |
| `python3 scripts/windows_native_probe.py fmt --seconds 600` | exit0 | 5.49s |
| `python3 scripts/windows_native_probe.py clippy --seconds 1200` | exit0 | 7.76s |
| `python3 scripts/windows_native_probe.py test --seconds 1200` | exit0, 116passed/1ignored | 13.12s |
| `python3 scripts/windows_native_probe.py build --seconds 1200` | exit0, release | 21.24s |

각 작업의 before/after 235개 Cargo/Rust/vendor 입력이 동일하며 네 작업 사이 입력도 동일했다. Cargo.lock SHA256은 `b49000da1a7f0a3b4c4697bceceb76c3d2b7789e36d10e2d678bfe9314ba9031`이다. release dlpad.exe는6823936bytes, SHA256 `2a0fbb2e1c5882e9feeb84eecc78e9313b72499a52b31063f768537fa3093e85`.

시험은 app15/core21/engine71(ignored1)/diagnostic3/Windows process6이다. 기존 Windows 활성화 IPC·file identity·publication recovery·process failure/cancel/parent crash 회귀를 포함한다. cfgLinux 전용 HTTP/media/disk/guardian 시험은 Windows에서0tests이며 완료로 확대하지 않는다.

범위는 기본 workspace의 Windows GNU 회귀와 새 Cargo resolver Windows edge다. CPU feature는 활성화하지 않았으며 CPU Windows GUI/AccessKit/IME/성능, MSVC, macOS는 미검증이다. 이전 Windows source snapshot의 증거를 재사용하지 않았다. 원시 logs/전체 입력 manifest/binary metadata는 windows-cpu-lock-default/result.json 및 같은 폴더에 보존했다.
