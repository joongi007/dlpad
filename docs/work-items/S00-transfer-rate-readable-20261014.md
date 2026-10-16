# S00: 전송 속도 자동 단위 표시

- 상태: in progress
- 분류: feature
- 우선순위: P0
- 요구사항: R01, R07
- 담당/수정 범위: display_rate.rs, accessible_metrics.rs는 advanced_units; 다운로드 caller·두 main 등록·중앙 catalog는 Root
- 선행 작업: v25 source671 검증 및 공급 동결 해제

## 사용자 결과

다운로드와 네트워크 수신·송신 속도를 크기에 맞는 B/s, KB/s, MB/s, GB/s로 읽는다. 단위는 속도 제한 입력과 같은 decimal 1000이며 디스크 용량 표시에는 영향을 주지 않는다.

## 계약과 범위

공유 API는 `format_bytes_per_second(Option<f64>) -> String`이다. 측정 없음·음수·NaN·무한대는 기존 `UNKNOWN`, 유효한 0은 `0 B/s`다. 양수는 한 자리 소수로 반올림하며 999.95 KB/s는 1.0 MB/s로 승격한다. 0.05 B/s 미만 양수는 `< 0.1 B/s`, 1e12 B/s 이상은 `≥ 1000 GB/s`로 제한하여 미세한 양수와 0을 구별하고 긴 숫자를 막는다. 이는 표시 정밀도이며 원 관측치를 바꾸지 않는다.

footer와 repaint display key가 동일 formatter를 사용한다. 샘플 간격·worker·네트워크 기준선·실패/준비/중단 정책·DB/schema·의존성은 변경하지 않는다. 기존 DESIGN의 compact 상태 표시와 중앙 catalog 단위를 따른다.

## 인수 조건

- 모든 단위 경계와 반올림 carry, unknown/zero/tiny/huge 입력을 검증한다.
- 실제 egui Context footer의 Label/value가 경계·무효 입력에 맞으며 display key와 일치한다.
- 같은 표시 값은 repaint를 추가하지 않고 단위 carry/unknown 변경은 반영한다.
- 기존 stale/error/stop 회귀를 유지한다. Windows 전체 및 실제 화면 검증은 Root가 수행한다.

## 검증과 handoff

Linux rustfmt 1.94.0 `--edition 2024` 두 파일 구문/서식 검사는 통과했다. 처음 edition2021 지정은 기존 let-chain 구문 때문에 실패했고 2024로 바로잡았다. Cargo/Windows 시험은 child에서 실행하지 않았다. 실제 Context 시험은 Root 실행 대기다. 제품 성능 strict18 원 실패는 그대로 보존하며 이 표시 변경이 CPU 예산을 해결했다고 주장하지 않는다.

원본 SHA, 변경 diff, 최종 SHA 및 필터는 `.local-tools/session-recovery-20261013/transfer-rate-readable-v1/`에 보존한다. 전체 UI·물리 입력·reader·IME 인수 완료를 뜻하지 않는다.
