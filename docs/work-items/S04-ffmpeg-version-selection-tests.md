# S04: FFmpeg verified selection process regressions

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당/수정 범위: diagnostic_cache; crates/engine/tests/ffmpeg_version_selection.rs, 이 문서만
- 선행 작업: root ToolManager::select_verified_ffmpeg

## 사용자 결과

FFmpeg Executable/ffprobe Auxiliary/Notice package의 두 실제 version process가 성공할 때만 active를 전환하고 mismatch/nonzero/mutation/foreign/pre-cancel은 prior active를 보존하는지 검증한다.

## 계약과 범위

Linux x86_64 synthetic Python shebang executables만 temp managed root에 생성한다. shell 없음, 네트워크/실제 FFmpeg/GUI/사용자 데이터 없음. argv가 정확히 [-version]인지 각 자식에서 assert한다. 현재 fixture native inventory metadata 형식은 기존 deno_version unit fixture와 동일하다. 프로젝트 공유/Cargo 변경 없음.

## 인수 조건

성공 exact first line의 두 version 조회 확인. ffprobe wrong/nonzero 및 원 executable mutation/foreign/pre-cancel 실패 이후 prior active 경로와 bytes 보존. sentinel/source 합성 외부 파일 보존. 실제 공급/codec/license/Windows 실행은 별도 gate이다.

## 검증과 handoff

작성/전용 fmt 후 root 통합 실행 대기. API 구현 전이므로 현재 compile/native 결과를 주장하지 않는다.
