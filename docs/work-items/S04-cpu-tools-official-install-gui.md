# S04 CPU 공식 고정 도구 설치·중단 GUI

- 상태: done
- 분류: verification
- 우선순위: P0
- 요구사항: R05
- 담당/수정 범위: stress_recovery; 신규 scripts/cpu_tools_install_probe.py, 이 문서 및 전용 evidence/validation.
- 선행 작업: CPU 관리도구 이식/기존 합성 선택 제거 검증, root frozen CPU166946…

## 사용자 결과

공식 pinned yt-dlp 설치 뒤 명시 Use와 inactive 제거를 native GUI로 확인한다. 실제 network transfer 중 설치 중단은 기존 active/외부 파일을 보존한다.

## 계약과 범위

ADR-0003, INTEGRATIONS, 현재 managed::candidate Linux pin2026.08.19/40446224B/SHA58162f9bfdc27458ea47bfcb311cf47028f17d8154a8bf7d689861d46399230a/공식GitHub URL 동일. 제품/shared helper/Cargo 수정 금지. 소유 tmp/Xvfb/privateDBus만 사용하며 media/site/account 실행과 user desktop/clipboard 금지. 공식파일 실행은 installer의 config/plugin 차단 --version과 같은 version 확인만. HttpsTransport proxy(None)로 local proxy 환경변수는 효과 없으므로 staging 실제 partial bytes를 관측하고 cached enabled Cancel action을 즉시 전달한다. 네트워크 제어가 필요한 경우 별도 계약 없이 제품정책을 바꾸지 않는다.

## 인수 조건

실제 install SHA/size/version/manifest/active, 명시 Use 입력path, inactive official 제거 후 선택된 합성 버전/외부 sentinel 보존. 실제 partial download(0<bytes<expected) 관측 뒤 Cancel UI action, active/retained bytes 보존/no publication/owned staging cleanup. 실제 native role/extents/disabled 상태/정상 종료와 worker/process 회수/raw/SHA 보존. endpoint/network 오류와 race를 성공으로 표현하지 않는다.

## 검증과 handoff

준비 단계다. root에 source/readiness 계약을 보고했다. 실제 외부 devtool download는 기존 사용자 승인 범위이며 공개 서버/보안변경/계정전송은 하지 않는다. GUI를 실행하기 전 정확 frozen SHA와 새 evidence 경로를 확인한다.

## 완료 근거와 남은 게이트

Linux 실제 install-1/cancel-1 GUI 각exit0/PASS. 공식40,446,224B/SHA/version/manifest/active와명시Use, disabledactiveRemove, inactiveofficialRemove를 확인했다. 별도freshcase에서실제partial16,384B관측후CancelAction으로기존active/외부bytes 보존/no publication/staging회수를 확인했다. 보존DBintegrity/active 및 host PID/start 부재를 직접검증했다. 제품변경/의존성추가/proxy·TLS 변경 없음. [원시 검증과 한계](../validation/cpu-tools-official-install-validation.md)를 따른다. Windows/macOS/FFmpeg/JS/runtime/배포공급/전체P0 완료는 이 bounded 수직 작업에서 주장하지 않는다.
