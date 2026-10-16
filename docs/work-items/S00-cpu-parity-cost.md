# S00: 통합 CPU 후보 비용 측정

- 상태: in-progress
- 분류: verification
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: root; 본 문서와 전용 accessible-performance/cpu-parity 증거
- 선행 작업: 동결 LinuxCPU522e release

## 사용자 결과

새 도구·이력·복귀 통합의 idle RSS/CPU/실행 파일 비용을 실제 측정하여 이전 snapshot과 구분한다.

## 계약과 범위

기존 accessible_performance_probe.py를 read-only 사용. owned Xvfb/privatebus/임시폴더/폰트/10초표본, 사용자 desktop fallback 없음. 도구·이력은lazy 미진입 기본화면 baseline이며 fullpeak/장시간/모든OS 인수와 구분.

## 인수 조건

동일 binary SHA/기동정상/RSS·CPU raw값/소유프로세스·임시폴더정리. 실제 GUI원시PNG검토, 예산을결과에맞춰변경하지않음.

## 검증과 handoff

측정 진행 중.
