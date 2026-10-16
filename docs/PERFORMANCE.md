# 성능 예산과 시험

아래는 S00에서 검증하고 수정할 **제안 목표**다. 달성된 수치가 아니다. 기능을 제외한 숫자만으로 가볍다고 주장하지 않는다.

| 항목 | 초기 예산 |
|---|---|
| 앱만의 압축 배포 크기 | 25 MiB 이하 목표, 외부 도구/설치 런타임 별도와 합계 공개 |
| UI 대기 상태 | resident/private memory 80 MiB 이하 목표 |
| 창 없는 작업 관리자 | 외부 도구 제외 40 MiB 이하 목표 |
| idle CPU | 기준 장비에서 단일 코어 기준 평균 1% 미만 |
| 첫 화면 | 기준 SSD 장비 warm start 1초 내 목표 |
| 활성 지표 갱신 | 약 1초 주기, background 5초; 애니메이션 때문에 영구 60 FPS 금지 |
| 8시간 작업 | 메모리의 시간 비례 증가 없음, 정상상태 대비 종료 직전 증가 원인 설명 |
| 작업 로그 | 작업당 5 MiB, 전체 50 MiB 초기 상한, rolling 보존 |
| 목록 | 10,000 entries, viewport 가상화·DB paging, 썸네일 32 MiB cache 상한 |

OS·CPU·RAM·GPU·배율·renderer·릴리스 옵션·dataset을 benchmark 결과에 함께 기록한다. Windows working set/private bytes, Linux RSS/PSS, macOS footprint를 동일 의미로 섞어 비교하지 않는다. 앱, 각 자식, 전체 트리, GPU 메모리를 가능하면 별도로 측정한다. 8시간 테스트 중 ffmpeg 재인코딩의 높은 CPU는 앱 idle 예산과 구별한다.

## 지표 정의

| 사용자 표시 | 의미 / 수집 |
|---|---|
| 다운로드 속도 | job에서 받은 미디어 bytes/time; yt-dlp 보고값 |
| 수신·송신 | 선택된 네트워크 interface counter delta; 다른 앱 트래픽 포함, VPN 중복 합산 주의 |
| 회선 속도 | 링크 협상값 또는 사용자 실행 벤치마크. 실제 다운로드 최고속도로 표기하지 않음 |
| 연결 상태 | interface 상태 + 요청 대상 접근 결과; 단일 probe로 인터넷 전체 판정 금지 |
| 미디어 처리 | FFmpeg speed×, output media time, 필요 시 fps; 불명확하면 ETA 미표시 |
| 디스크 I/O | 지원되는 프로세스/볼륨 counter delta, 캐시와 물리 쓰기 구별 |
| 여유 공간 | 대상 volume 잔여 bytes. 병합 입력+출력 동시 존재와 재인코딩 여유 고려 |

네트워크·디스크 최고 처리량 측정은 P1 선택 진단. 사용자 시작, 대상·트래픽·쓰기량 한도, 취소를 제공한다. 상시 benchmark나 초기 실행 자동 speed test는 하지 않는다. counter가 없을 때 0으로 속이지 않는다.

## 필수 시나리오

1. 빈 목록, 10,000 파일, 큰 한글 제목, 대기/스크롤/창 숨김의 메모리와 CPU.
2. 로컬 제어 서버로 range/HLS, 느린 응답, 연결 단절, URL 만료, 용량 알 수 없음, 429/5xx를 재현.
3. 8시간 이상 실시간 soak와 큰 바이트 수(4 GiB 이상), sleep/resume, 디스크 부족, 대상 볼륨 제거, 종료 중 crash.
4. download→merge→rename 각 경계 강제 종료 후 복구. 중복 완료·고아 프로세스·파일 유실 없음.
5. OS별 한글 IME, 스크린리더, multi-DPI, 트레이 유무, 외부/관리 도구 설치·삭제.

PR에서는 짧은 fixture와 가속된 상태 시험, nightly/릴리스 후보에서는 실제 8시간 soak를 실행한다. 실제 사이트 smoke는 네트워크 가변성 때문에 별도 결과로 기록한다. 합법적으로 시험 가능한 소스로 YouTube·치지직 VOD를 수동 검증한다.

## 빠른 8시간 타임라인과 실시간 시험의 구분

개발 반복에서는 [8시간 합성 타임라인](validation/fast-eight-hour-timeline.md)을 먼저 실행한다. 분당1frame 색상 영상의 duration28800초를 실제ffprobe로 확인하고 다운로드·끝부분구간·decode·DB재열기를약23초에시험했다. 영상duration·파일크기·벽시계시간은각각다른축이다. 시간기반상태전이는기존deterministic unit/faultfixture로검사하며메모리/FD누수·장시간멈춤·sleep·실제사이트·4GiB byte축은독립게이트로유지한다. 작은fixture를일반영상성능이나8시간wallclock성공으로주장하지않는다.

## 평균 목표와 고정 구간 진단의 구분

문서의 idle CPU 제안 목표는 단일 코어 기준 **평균 1% 미만**이다. 3×60초 측정에서 추가로 사용하는 18개 고정10초 구간의 각각1% 미만 기준은 피크를 발견하기 위한 더 엄격한 진단 gate다. 구간 실패를 평균 성공으로 바꾸지 않으며 모든 구간·각60초 평균·측정 환경과 source/artifact를 함께 공개한다. 첫 구간 제거·측정 시간 연장·진단 feature 결과로 기본 결과 대체는 하지 않는다. 목표 달성만으로 전체 P0/stack/물리 UI 인수를 주장하지 않는다.

2026-10-13 source654/F42 기본 빌드의 세60초 평균은0.520171%,0.676708%,0.598196%다. 같은 측정의18구간은 CPU16PASS/2FAIL(1.2483007755%,1.5665082560%)이며 memory18PASS다. 평균 제안 목표 아래라는 관측과 strict 진단 실패를 동시에 유지한다. quiet host·scan 부재·worker CPU 인과 귀속은 미검증이다. 후속 source655 UI 통합본에는 이 수치를 소급하지 않는다. [원 실패 검토](validation/evidence/current-default18-idle-cost-20261013/root-review.json).

후속source655/8ab7 기본빌드: 세60초평균0.5988757384%,0.5987914980%,0.5723074711%, strict18 CPU17PASS/1FAIL(1회차5구간1.0861521808%), memory18PASS다. 기본/진단둘다자연종료/ownedJob0이며이차이의원인·quiet reference host는미검증이다. 새기본평균관측도전체P0/stack인수가아니다. [최신 기본실패 검토](validation/evidence/current-ui-feedback-default18-cost-20261013/root-review.json).

2026-10-14 v22/source669 기본 결과: 보고된세60초평균0.598640/0.884629/0.650723%이며 원18구간 counter를 독립 재계산한 가중평균은0.598679/0.884690/0.650767%다. 평균 목표 아래라는 관측과 strict18구간CPU16PASS/2FAIL·메모리18PASS를 구분한다. 두FAIL은1.093468/1.092684%이며 thread churn 때문에 완전한원인귀속은 미확정이다. 최종60초counter가원자료에없어 보고평균의exactraw검증을주장하지않는다. 세앱자연0·Job0, 읽기전용3DB/30,000원fixture와6PNG를확인했다. [현재 독립 검토](validation/evidence/current-ui-feedback-v22-default18-cost-20261014/root-review.json).


2026-10-16 v32/source675 기본 빌드: 실제 60초×3회 평균은0.5202217476%/0.5982905234%/0.4162663059%, strict18구간 CPU17 PASS/1 FAIL(1.2475643801%)·memory18 PASS다. 합산 시간180.1952167초·원 CPU9,218,750 ticks(100ns)로 재계산한 시간 가중 평균은0.5115979308%다. 원 첫 두 회는 그대로 두고 UIA setup에서 중단된 세 번째만 다시 준비했다. setup 조회만 보완한 혼합 observer 버전이며 측정 prefix는 바이트 동일하다. 중단된 원 시도의 owned termination과 원 strict 실패를 보존한다. Root가 실제 Windows 파일30,000개 payload/native ID/mtime·readonly3DB·6PNG와 원 counter를 대조했다. quiet host와 worker 작업의 완전한 인과 귀속은 미검증이다. [원 검토](validation/evidence/current32-default18-composite-root-20261016/root-review.json).

2026-10-16 v33/source675 기본 빌드의 새3×60초 관측: 원final counter 재계산 평균은0.5723777454%/0.4941536640%/0.4941599523%, 최대 sampled whole-Job 작업집합80,871,424B(77.125MiB)다. 추가strict18구간 CPU17PASS/1FAIL(3회차3구간1.0920442160%)·memory18PASS이며 원collectorFAIL/Python1/nativeidle2를 그대로 보존했다. 세 measured앱은 자연0·Job0이고 Root는 실제Windows30,000개3byte파일의 ID/mtime·readonly3DB·6PNG·원18구간/60초counter를 독립 대조했다. 모든 경쟁 dlpad probe 종료 후 한 번 측정했으며 첫구간제거·threshold변경·재시도로실패없애기0이다. quiet host/비원자counter/worker 귀속의 한계와 물리·전체P0 미인수는 유지한다. [현재33 원 검토](validation/evidence/current33-default18-native-root-review-20261016/root-review.json).
